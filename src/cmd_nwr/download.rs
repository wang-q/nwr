use clap::*;
use log::info;
use simplelog::*;
use std::fs::File;
use std::io;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("download")
        .about("Download the latest releases of `taxdump` and assembly reports")
        .after_help(include_str!("../../docs/help/download.md"))
        .arg(
            Arg::new("host")
                .long("host")
                .num_args(1)
                .default_value("ftp.ncbi.nih.gov:21")
                .help("NCBI FTP Host:Port"),
        )
        .arg(
            Arg::new("tx")
                .long("tx")
                .num_args(1)
                .default_value("/pub/taxonomy")
                .help("NCBI FTP Path of taxonomy"),
        )
        .arg(
            Arg::new("ar")
                .long("ar")
                .num_args(1)
                .default_value("/genomes/ASSEMBLY_REPORTS")
                .help("NCBI FTP Path of assembly reports"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    let nwrdir = nwr::nwr_path()?;
    let tarball = nwrdir.join("taxdump.tar.gz");
    let ar_refseq = nwrdir.join("assembly_summary_refseq.txt");
    let ar_genbank = nwrdir.join("assembly_summary_genbank.txt");

    // download taxdump
    info!(
        "==> Downloading from {} ...",
        args.get_one::<String>("host").unwrap()
    );
    if std::path::Path::new(&tarball).exists() {
        info!("Skipping, {} exists", tarball.to_string_lossy());
    } else {
        info!("Connecting...");
        let mut conn = ftp::FtpStream::connect(args.get_one::<String>("host").unwrap())?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        conn.cwd(args.get_one::<String>("tx").unwrap())?;
        info!("Remote directory: {}", conn.pwd().unwrap());

        info!("Retrieving MD5 file...");
        let mut file = File::create(nwrdir.join("taxdump.tar.gz.md5"))?;
        let mut cursor = conn.simple_retr("taxdump.tar.gz.md5")?;
        io::copy(&mut cursor, &mut file)?;

        info!("Retrieving {}...", "taxdump.tar.gz");
        conn.retr("taxdump.tar.gz", |stream| {
            let mut file = match File::create(&tarball) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file).map_err(ftp::FtpError::ConnectionError)
        })?;

        conn.quit()?;
        info!("End connection.");
    }

    // check
    info!("==> Checking...");
    {
        let mut file = File::open(&tarball)?;
        let mut hasher = md5::Context::new();
        info!("Computing MD5 sum...");
        io::copy(&mut file, &mut hasher)?;
        let digest = format!("{:x}", hasher.compute());

        let mut ncbi_digest =
            std::fs::read_to_string(nwrdir.join("taxdump.tar.gz.md5"))?;
        ncbi_digest.truncate(32);

        if digest != ncbi_digest {
            return Err(anyhow::anyhow!(
                "MD5 check failed. Expected: {}, Computed: {}",
                ncbi_digest,
                digest
            ));
        } else {
            info!("MD5 sum passed");
        }
    }

    // extract
    info!("==> Extracting...");
    {
        let tar_gz = File::open(&tarball)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);

        let mut archive = tar::Archive::new(tar);
        archive.unpack(nwrdir)?;
    }

    // assembly reports
    info!(
        "==> Downloading from {} ...",
        args.get_one::<String>("host").unwrap()
    );
    if std::path::Path::new(&ar_refseq).exists()
        && std::path::Path::new(&ar_genbank).exists()
    {
        info!(
            "Skipping, {} & {} exist",
            ar_refseq.to_string_lossy(),
            ar_genbank.to_string_lossy()
        );
    } else {
        info!("Connecting...");
        let mut conn = ftp::FtpStream::connect(args.get_one::<String>("host").unwrap())?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        conn.cwd(args.get_one::<String>("ar").unwrap())?;
        info!("Remote directory: {}", conn.pwd().unwrap());

        info!("Retrieving {}...", "assembly_summary_refseq.txt");
        conn.retr("assembly_summary_refseq.txt", |stream| {
            let mut file = match File::create(&ar_refseq) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file).map_err(ftp::FtpError::ConnectionError)
        })?;

        info!("Retrieving {}...", "assembly_summary_genbank.txt");
        conn.retr("assembly_summary_genbank.txt", |stream| {
            let mut file = match File::create(&ar_genbank) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file).map_err(ftp::FtpError::ConnectionError)
        })?;

        conn.quit()?;
        info!("End connection.");
    }

    info!("File sizes:");
    for f in &[tarball, ar_refseq, ar_genbank] {
        info!(
            "{}\t{}",
            f.to_string_lossy(),
            readable(f.metadata().unwrap().len().to_string())
        );
    }

    Ok(())
}

fn readable(n: String) -> String {
    let mut c = String::new();

    for (i, char) in n.chars().rev().enumerate() {
        if i % 3 == 0 && i != 0 {
            c.insert(0, ',');
        }
        c.insert(0, char);
    }

    c
}

/// Verify MD5 checksum of a file against expected digest
#[allow(dead_code)]
fn verify_md5(
    file_path: &std::path::Path,
    expected_digest: &str,
) -> anyhow::Result<bool> {
    let mut file = File::open(file_path)?;
    let mut hasher = md5::Context::new();
    io::copy(&mut file, &mut hasher)?;
    let digest = format!("{:x}", hasher.compute());
    Ok(digest == expected_digest)
}

/// Extract tar.gz file to destination directory
#[allow(dead_code)]
fn extract_tarball(
    tarball_path: &std::path::Path,
    dest_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let tar_gz = File::open(tarball_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(dest_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_readable_small_number() {
        assert_eq!(readable("123".to_string()), "123");
    }

    #[test]
    fn test_readable_thousands() {
        assert_eq!(readable("1234".to_string()), "1,234");
    }

    #[test]
    fn test_readable_millions() {
        assert_eq!(readable("1234567".to_string()), "1,234,567");
    }

    #[test]
    fn test_readable_zero() {
        assert_eq!(readable("0".to_string()), "0");
    }

    #[test]
    fn test_readable_large_number() {
        assert_eq!(readable("1234567890".to_string()), "1,234,567,890");
    }

    #[test]
    fn test_readable_empty_string() {
        assert_eq!(readable("".to_string()), "");
    }

    #[test]
    fn test_readable_exact_boundary() {
        // Test exactly at thousand boundaries
        assert_eq!(readable("1000".to_string()), "1,000");
        assert_eq!(readable("1000000".to_string()), "1,000,000");
    }

    #[test]
    fn test_verify_md5_correct() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();
        drop(file);

        // MD5 of "Hello, World!"
        let result = verify_md5(&file_path, "65a8e27d8879283831b664bd8b7f0ad4");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_md5_incorrect() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();
        drop(file);

        // Wrong MD5
        let result = verify_md5(&file_path, "00000000000000000000000000000000");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_md5_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = verify_md5(&file_path, "any_digest");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_tarball() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let temp_dir = TempDir::new().unwrap();
        let tarball_path = temp_dir.path().join("test.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir(&extract_dir).unwrap();

        // Create a simple tar.gz file
        {
            let tar_gz = File::create(&tarball_path).unwrap();
            let enc = GzEncoder::new(tar_gz, Compression::default());
            let mut tar = Builder::new(enc);

            // Add a test file to the archive
            let test_content = b"Hello from tar archive!";
            let mut header = tar::Header::new_gnu();
            header.set_path("test_file.txt").unwrap();
            header.set_size(test_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, &test_content[..]).unwrap();
            // Builder will be dropped here, finishing the archive
        }

        // Extract the tarball
        let result = extract_tarball(&tarball_path, &extract_dir);
        assert!(result.is_ok(), "Failed to extract tarball: {:?}", result);

        // Verify the extracted file
        let extracted_file = extract_dir.join("test_file.txt");
        assert!(extracted_file.exists(), "Extracted file does not exist");
        let content = std::fs::read_to_string(extracted_file).unwrap();
        assert_eq!(content, "Hello from tar archive!");
    }

    #[test]
    fn test_extract_tarball_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let tarball_path = temp_dir.path().join("nonexistent.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");

        let result = extract_tarball(&tarball_path, &extract_dir);
        assert!(result.is_err());
    }
}

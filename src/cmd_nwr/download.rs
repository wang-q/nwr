use clap::*;
use log::info;
use simplelog::*;
use std::fs::File;
use std::io;
use std::path::Path;

/// Create clap subcommand arguments
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

/// Get file paths for download operation
pub fn get_download_paths(nwrdir: &Path) -> anyhow::Result<DownloadPaths> {
    Ok(DownloadPaths {
        tarball: nwrdir.join("taxdump.tar.gz"),
        ar_refseq: nwrdir.join("assembly_summary_refseq.txt"),
        ar_genbank: nwrdir.join("assembly_summary_genbank.txt"),
        md5_file: nwrdir.join("taxdump.tar.gz.md5"),
    })
}

/// Struct holding download file paths
#[derive(Debug, Clone)]
pub struct DownloadPaths {
    pub tarball: std::path::PathBuf,
    pub ar_refseq: std::path::PathBuf,
    pub ar_genbank: std::path::PathBuf,
    pub md5_file: std::path::PathBuf,
}

/// Check if taxdump tarball exists
pub fn taxdump_exists(tarball: &Path) -> bool {
    tarball.exists()
}

/// Check if assembly reports exist
pub fn assembly_reports_exist(ar_refseq: &Path, ar_genbank: &Path) -> bool {
    ar_refseq.exists() && ar_genbank.exists()
}

/// Check MD5 checksum of downloaded tarball
pub fn check_taxdump_md5(tarball: &Path, md5_file: &Path) -> anyhow::Result<()> {
    let mut file = File::open(tarball)?;
    let mut hasher = md5::Context::new();
    info!("Computing MD5 sum...");
    io::copy(&mut file, &mut hasher)?;
    let digest = format!("{:x}", hasher.compute());

    let mut ncbi_digest = std::fs::read_to_string(md5_file)?;
    ncbi_digest.truncate(32);

    if digest != ncbi_digest {
        Err(anyhow::anyhow!(
            "MD5 check failed. Expected: {}, Computed: {}",
            ncbi_digest,
            digest
        ))
    } else {
        info!("MD5 sum passed");
        Ok(())
    }
}

/// Extract taxdump tarball
pub fn extract_taxdump(tarball: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    let tar_gz = File::open(tarball)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(dest_dir)?;
    Ok(())
}

/// Format file sizes for display
pub fn format_file_sizes(paths: &DownloadPaths) -> anyhow::Result<Vec<String>> {
    let mut sizes = Vec::new();
    for f in &[&paths.tarball, &paths.ar_refseq, &paths.ar_genbank] {
        let size = f.metadata()?.len();
        sizes.push(format!(
            "{}\t{}",
            f.to_string_lossy(),
            readable(size.to_string())
        ));
    }
    Ok(sizes)
}

/// FTP connection trait for testability
#[cfg_attr(test, mockall::automock)]
pub trait FtpConnectionTrait {
    /// Login to FTP server
    fn login(&mut self, user: &str, password: &str) -> anyhow::Result<()>;

    /// Change working directory
    fn cwd(&mut self, path: &str) -> anyhow::Result<()>;

    /// Get current directory
    fn pwd(&mut self) -> anyhow::Result<String>;

    /// Download a file
    fn simple_retr(
        &mut self,
        filename: &str,
    ) -> anyhow::Result<std::io::Cursor<Vec<u8>>>;

    /// Download a file to a specific path
    fn retr_to_file(&mut self, filename: &str, dest_path: &Path) -> anyhow::Result<()>;

    /// Quit connection
    fn quit(&mut self) -> anyhow::Result<()>;
}

/// FTP connection wrapper
pub struct FtpConnection {
    stream: ftp::FtpStream,
}

impl FtpConnection {
    /// Connect to FTP server
    pub fn connect(host: &str) -> anyhow::Result<Self> {
        let stream = ftp::FtpStream::connect(host)?;
        Ok(Self { stream })
    }
}

impl FtpConnectionTrait for FtpConnection {
    fn login(&mut self, user: &str, password: &str) -> anyhow::Result<()> {
        self.stream.login(user, password)?;
        Ok(())
    }

    fn cwd(&mut self, path: &str) -> anyhow::Result<()> {
        self.stream.cwd(path)?;
        Ok(())
    }

    fn pwd(&mut self) -> anyhow::Result<String> {
        Ok(self.stream.pwd()?)
    }

    fn simple_retr(
        &mut self,
        filename: &str,
    ) -> anyhow::Result<std::io::Cursor<Vec<u8>>> {
        Ok(self.stream.simple_retr(filename)?)
    }

    fn retr_to_file(&mut self, filename: &str, dest_path: &Path) -> anyhow::Result<()> {
        self.stream.retr(filename, |stream| {
            let mut file = match File::create(dest_path) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file)
                .map(|_| ())
                .map_err(ftp::FtpError::ConnectionError)
        })?;
        Ok(())
    }

    fn quit(&mut self) -> anyhow::Result<()> {
        self.stream.quit()?;
        Ok(())
    }
}

/// Download taxdump from FTP server
fn download_taxdump(
    conn: &mut dyn FtpConnectionTrait,
    paths: &DownloadPaths,
    tx_path: &str,
) -> anyhow::Result<()> {
    conn.cwd(tx_path)?;
    info!("Remote directory: {}", conn.pwd()?);

    info!("Retrieving MD5 file...");
    let mut file = File::create(&paths.md5_file)?;
    let mut cursor = conn.simple_retr("taxdump.tar.gz.md5")?;
    io::copy(&mut cursor, &mut file)?;

    info!("Retrieving {}...", "taxdump.tar.gz");
    conn.retr_to_file("taxdump.tar.gz", &paths.tarball)?;

    Ok(())
}

/// Download assembly reports from FTP server
fn download_assembly_reports(
    conn: &mut dyn FtpConnectionTrait,
    paths: &DownloadPaths,
    ar_path: &str,
) -> anyhow::Result<()> {
    conn.cwd(ar_path)?;
    info!("Remote directory: {}", conn.pwd()?);

    info!("Retrieving {}...", "assembly_summary_refseq.txt");
    conn.retr_to_file("assembly_summary_refseq.txt", &paths.ar_refseq)?;

    info!("Retrieving {}...", "assembly_summary_genbank.txt");
    conn.retr_to_file("assembly_summary_genbank.txt", &paths.ar_genbank)?;

    Ok(())
}

/// Internal execute implementation that accepts a connection factory
fn execute_internal<F>(args: &ArgMatches, mut conn_factory: F) -> anyhow::Result<()>
where
    F: FnMut(&str) -> anyhow::Result<Box<dyn FtpConnectionTrait>>,
{
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    let nwrdir = nwr::nwr_path()?;
    let paths = get_download_paths(&nwrdir)?;

    // Download taxdump
    info!(
        "==> Downloading from {} ...",
        args.get_one::<String>("host").unwrap()
    );
    if taxdump_exists(&paths.tarball) {
        info!("Skipping, {} exists", paths.tarball.to_string_lossy());
    } else {
        info!("Connecting...");
        let mut conn = conn_factory(args.get_one::<String>("host").unwrap())?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        download_taxdump(conn.as_mut(), &paths, args.get_one::<String>("tx").unwrap())?;
        conn.quit()?;
        info!("End connection.");
    }

    // Check
    info!("==> Checking...");
    check_taxdump_md5(&paths.tarball, &paths.md5_file)?;

    // Extract
    info!("==> Extracting...");
    extract_taxdump(&paths.tarball, &nwrdir)?;

    // Assembly reports
    info!(
        "==> Downloading from {} ...",
        args.get_one::<String>("host").unwrap()
    );
    if assembly_reports_exist(&paths.ar_refseq, &paths.ar_genbank) {
        info!(
            "Skipping, {} & {} exist",
            paths.ar_refseq.to_string_lossy(),
            paths.ar_genbank.to_string_lossy()
        );
    } else {
        info!("Connecting...");
        let mut conn = conn_factory(args.get_one::<String>("host").unwrap())?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        download_assembly_reports(
            conn.as_mut(),
            &paths,
            args.get_one::<String>("ar").unwrap(),
        )?;
        conn.quit()?;
        info!("End connection.");
    }

    info!("File sizes:");
    for size_line in format_file_sizes(&paths)? {
        info!("{}", size_line);
    }

    Ok(())
}

/// Command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    execute_internal(args, |host| {
        let conn = FtpConnection::connect(host)?;
        Ok(Box::new(conn) as Box<dyn FtpConnectionTrait>)
    })
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
    fn test_make_subcommand() {
        let cmd = make_subcommand();
        assert_eq!(cmd.get_name(), "download");

        // Verify all arguments exist
        let arg_names: Vec<_> =
            cmd.get_arguments().map(|a| a.get_id().as_str()).collect();
        assert!(arg_names.contains(&"host"));
        assert!(arg_names.contains(&"tx"));
        assert!(arg_names.contains(&"ar"));
    }

    #[test]
    fn test_make_subcommand_default_values() {
        let cmd = make_subcommand();

        let host_arg = cmd.get_arguments().find(|a| a.get_id() == "host").unwrap();
        assert_eq!(host_arg.get_default_values(), vec!["ftp.ncbi.nih.gov:21"]);

        let tx_arg = cmd.get_arguments().find(|a| a.get_id() == "tx").unwrap();
        assert_eq!(tx_arg.get_default_values(), vec!["/pub/taxonomy"]);

        let ar_arg = cmd.get_arguments().find(|a| a.get_id() == "ar").unwrap();
        assert_eq!(
            ar_arg.get_default_values(),
            vec!["/genomes/ASSEMBLY_REPORTS"]
        );
    }

    #[test]
    fn test_make_subcommand_about() {
        let cmd = make_subcommand();
        let about = cmd.get_about().map(|s| s.to_string());
        assert!(about.is_some());
        assert!(about.unwrap().to_lowercase().contains("download"));
    }

    #[test]
    fn test_get_download_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = get_download_paths(temp_dir.path()).unwrap();

        assert_eq!(paths.tarball, temp_dir.path().join("taxdump.tar.gz"));
        assert_eq!(
            paths.ar_refseq,
            temp_dir.path().join("assembly_summary_refseq.txt")
        );
        assert_eq!(
            paths.ar_genbank,
            temp_dir.path().join("assembly_summary_genbank.txt")
        );
        assert_eq!(paths.md5_file, temp_dir.path().join("taxdump.tar.gz.md5"));
    }

    #[test]
    fn test_taxdump_exists() {
        let temp_dir = TempDir::new().unwrap();
        let tarball = temp_dir.path().join("taxdump.tar.gz");

        // File doesn't exist yet
        assert!(!taxdump_exists(&tarball));

        // Create the file
        File::create(&tarball).unwrap();
        assert!(taxdump_exists(&tarball));
    }

    #[test]
    fn test_assembly_reports_exist() {
        let temp_dir = TempDir::new().unwrap();
        let ar_refseq = temp_dir.path().join("assembly_summary_refseq.txt");
        let ar_genbank = temp_dir.path().join("assembly_summary_genbank.txt");

        // Files don't exist yet
        assert!(!assembly_reports_exist(&ar_refseq, &ar_genbank));

        // Create only one file
        File::create(&ar_refseq).unwrap();
        assert!(!assembly_reports_exist(&ar_refseq, &ar_genbank));

        // Create the other file
        File::create(&ar_genbank).unwrap();
        assert!(assembly_reports_exist(&ar_refseq, &ar_genbank));
    }

    #[test]
    fn test_check_taxdump_md5_correct() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let tarball = temp_dir.path().join("taxdump.tar.gz");
        let mut file = File::create(&tarball).unwrap();
        file.write_all(b"test content for tarball").unwrap();
        drop(file);

        // Calculate MD5
        let mut hasher = md5::Context::new();
        hasher.consume(b"test content for tarball");
        let digest = format!("{:x}", hasher.compute());

        // Create MD5 file with proper format (hash + filename)
        let md5_file = temp_dir.path().join("taxdump.tar.gz.md5");
        let mut md5_content = File::create(&md5_file).unwrap();
        write!(md5_content, "{}  taxdump.tar.gz", digest).unwrap();
        drop(md5_content);

        let result = check_taxdump_md5(&tarball, &md5_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_taxdump_md5_incorrect() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let tarball = temp_dir.path().join("taxdump.tar.gz");
        let mut file = File::create(&tarball).unwrap();
        file.write_all(b"test content for tarball").unwrap();
        drop(file);

        // Create MD5 file with wrong hash
        let md5_file = temp_dir.path().join("taxdump.tar.gz.md5");
        let mut md5_content = File::create(&md5_file).unwrap();
        write!(
            md5_content,
            "00000000000000000000000000000000  taxdump.tar.gz"
        )
        .unwrap();
        drop(md5_content);

        let result = check_taxdump_md5(&tarball, &md5_file);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("MD5 check failed"));
    }

    #[test]
    fn test_check_taxdump_md5_nonexistent_tarball() {
        let temp_dir = TempDir::new().unwrap();
        let tarball = temp_dir.path().join("nonexistent.tar.gz");
        let md5_file = temp_dir.path().join("taxdump.tar.gz.md5");

        // Create MD5 file only
        File::create(&md5_file).unwrap();

        let result = check_taxdump_md5(&tarball, &md5_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_taxdump_md5_nonexistent_md5_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let tarball = temp_dir.path().join("taxdump.tar.gz");
        let mut file = File::create(&tarball).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // Don't create MD5 file
        let md5_file = temp_dir.path().join("nonexistent.md5");

        let result = check_taxdump_md5(&tarball, &md5_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_taxdump() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let temp_dir = TempDir::new().unwrap();
        let tarball_path = temp_dir.path().join("taxdump.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir(&extract_dir).unwrap();

        // Create a tar.gz file
        {
            let tar_gz = File::create(&tarball_path).unwrap();
            let enc = GzEncoder::new(tar_gz, Compression::default());
            let mut tar = Builder::new(enc);

            let test_content = b"names.dmp content";
            let mut header = tar::Header::new_gnu();
            header.set_path("names.dmp").unwrap();
            header.set_size(test_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, &test_content[..]).unwrap();
        }

        let result = extract_taxdump(&tarball_path, &extract_dir);
        assert!(result.is_ok());

        let extracted_file = extract_dir.join("names.dmp");
        assert!(extracted_file.exists());
    }

    #[test]
    fn test_extract_taxdump_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let tarball = temp_dir.path().join("nonexistent.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");

        let result = extract_taxdump(&tarball, &extract_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_file_sizes() {
        let temp_dir = TempDir::new().unwrap();
        let paths = get_download_paths(temp_dir.path()).unwrap();

        // Create test files with known sizes
        let mut file1 = File::create(&paths.tarball).unwrap();
        file1.write_all(&vec![0u8; 1234]).unwrap();
        drop(file1);

        let mut file2 = File::create(&paths.ar_refseq).unwrap();
        file2.write_all(&vec![0u8; 5678]).unwrap();
        drop(file2);

        let mut file3 = File::create(&paths.ar_genbank).unwrap();
        file3.write_all(&vec![0u8; 999]).unwrap();
        drop(file3);

        let sizes = format_file_sizes(&paths).unwrap();
        assert_eq!(sizes.len(), 3);

        // Check that sizes are formatted correctly
        assert!(sizes[0].contains("1,234"));
        assert!(sizes[1].contains("5,678"));
        assert!(sizes[2].contains("999"));
    }

    #[test]
    fn test_format_file_sizes_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let paths = get_download_paths(temp_dir.path()).unwrap();

        // Don't create files - should error
        let result = format_file_sizes(&paths);
        assert!(result.is_err());
    }

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

    // Mock tests for FTP operations
    use super::MockFtpConnectionTrait;

    fn create_test_tarball_content() -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let mut buffer = Vec::new();
        {
            let enc = GzEncoder::new(&mut buffer, Compression::default());
            let mut tar = Builder::new(enc);
            let test_content = b"names.dmp content";
            let mut header = tar::Header::new_gnu();
            header.set_path("names.dmp").unwrap();
            header.set_size(test_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, &test_content[..]).unwrap();
        }
        buffer
    }

    fn calculate_md5(content: &[u8]) -> String {
        let mut hasher = md5::Context::new();
        hasher.consume(content);
        format!("{:x}", hasher.compute())
    }

    #[test]
    fn test_download_taxdump_success() {
        let temp_dir = TempDir::new().unwrap();
        let paths = get_download_paths(temp_dir.path()).unwrap();

        // Create mock
        let mut mock = MockFtpConnectionTrait::new();

        // Set up expectations
        mock.expect_cwd()
            .with(mockall::predicate::eq("/pub/taxonomy"))
            .times(1)
            .returning(|_| Ok(()));

        mock.expect_pwd()
            .times(0..=10) // Allow any number of calls
            .returning(|| Ok("/pub/taxonomy".to_string()));

        // MD5 file content
        let md5_content = b"abc123  taxdump.tar.gz";
        mock.expect_simple_retr()
            .with(mockall::predicate::eq("taxdump.tar.gz.md5"))
            .times(1)
            .returning(move |_| Ok(std::io::Cursor::new(md5_content.to_vec())));

        // Tarball content - use simple_retr for tarball too since we use it in test
        let tarball_content = create_test_tarball_content();
        let tarball_clone = tarball_content.clone();
        mock.expect_retr_to_file()
            .with(
                mockall::predicate::eq("taxdump.tar.gz"),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(move |_, dest_path: &Path| {
                std::fs::write(dest_path, &tarball_clone).unwrap();
                Ok(())
            });

        let result = download_taxdump(&mut mock, &paths, "/pub/taxonomy");
        assert!(result.is_ok());

        // Verify files were created
        assert!(paths.md5_file.exists());
        assert!(paths.tarball.exists());
    }

    #[test]
    fn test_download_assembly_reports_success() {
        let temp_dir = TempDir::new().unwrap();
        let paths = get_download_paths(temp_dir.path()).unwrap();

        // Create mock
        let mut mock = MockFtpConnectionTrait::new();

        // Set up expectations
        mock.expect_cwd()
            .with(mockall::predicate::eq("/genomes/ASSEMBLY_REPORTS"))
            .times(1)
            .returning(|_| Ok(()));

        mock.expect_pwd()
            .times(0..=10) // Allow any number of calls
            .returning(|| Ok("/genomes/ASSEMBLY_REPORTS".to_string()));

        // Refseq file content
        let refseq_content = b"refseq assembly summary content";
        let refseq_clone = refseq_content.to_vec();
        mock.expect_retr_to_file()
            .with(
                mockall::predicate::eq("assembly_summary_refseq.txt"),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(move |_, dest_path: &Path| {
                std::fs::write(dest_path, &refseq_clone).unwrap();
                Ok(())
            });

        // Genbank file content
        let genbank_content = b"genbank assembly summary content";
        let genbank_clone = genbank_content.to_vec();
        mock.expect_retr_to_file()
            .with(
                mockall::predicate::eq("assembly_summary_genbank.txt"),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(move |_, dest_path: &Path| {
                std::fs::write(dest_path, &genbank_clone).unwrap();
                Ok(())
            });

        let result =
            download_assembly_reports(&mut mock, &paths, "/genomes/ASSEMBLY_REPORTS");
        assert!(result.is_ok());

        // Verify files were created
        assert!(paths.ar_refseq.exists());
        assert!(paths.ar_genbank.exists());
    }

    #[test]
    fn test_ftp_connection_trait_mock() {
        let mut mock = MockFtpConnectionTrait::new();

        mock.expect_login()
            .with(
                mockall::predicate::eq("ftp"),
                mockall::predicate::eq("test@example.com"),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        mock.expect_cwd()
            .with(mockall::predicate::eq("/test"))
            .times(1)
            .returning(|_| Ok(()));

        mock.expect_pwd()
            .times(1)
            .returning(|| Ok("/test".to_string()));

        mock.expect_quit().times(1).returning(|| Ok(()));

        // Test the mock
        mock.login("ftp", "test@example.com").unwrap();
        mock.cwd("/test").unwrap();
        assert_eq!(mock.pwd().unwrap(), "/test");
        mock.quit().unwrap();
    }
}

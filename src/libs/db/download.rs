use log::info;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// File paths used by the download operation.
#[derive(Debug, Clone)]
pub struct DownloadPaths {
    /// Local path for the taxonomy dump tarball.
    pub tarball: PathBuf,
    /// Local path for the RefSeq assembly summary.
    pub ar_refseq: PathBuf,
    /// Local path for the GenBank assembly summary.
    pub ar_genbank: PathBuf,
    /// Local path for the tarball MD5 checksum file.
    pub md5_file: PathBuf,
}

/// Build the set of download file paths under `nwrdir`.
pub fn get_download_paths(nwrdir: &Path) -> anyhow::Result<DownloadPaths> {
    Ok(DownloadPaths {
        tarball: nwrdir.join("taxdump.tar.gz"),
        ar_refseq: nwrdir.join("assembly_summary_refseq.txt"),
        ar_genbank: nwrdir.join("assembly_summary_genbank.txt"),
        md5_file: nwrdir.join("taxdump.tar.gz.md5"),
    })
}

/// Check if the taxdump tarball exists locally.
pub fn taxdump_exists(tarball: &Path) -> bool {
    tarball.exists()
}

/// Check if both assembly report files exist locally.
pub fn assembly_reports_exist(ar_refseq: &Path, ar_genbank: &Path) -> bool {
    ar_refseq.exists() && ar_genbank.exists()
}

/// Verify the MD5 checksum of a downloaded taxdump tarball.
pub fn check_taxdump_md5(tarball: &Path, md5_file: &Path) -> anyhow::Result<()> {
    let mut file = File::open(tarball)?;
    let mut hasher = md5::Context::new();
    info!("Computing MD5 sum...");
    io::copy(&mut file, &mut hasher)?;
    let digest = format!("{:x}", hasher.compute());

    let ncbi_digest = std::fs::read_to_string(md5_file)?
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("MD5 file is empty: {}", md5_file.display()))?
        .to_lowercase();

    if ncbi_digest.len() != 32 {
        return Err(anyhow::anyhow!(
            "MD5 file has invalid checksum length: expected 32, got {} in {}",
            ncbi_digest.len(),
            md5_file.display()
        ));
    }

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

/// Extract a taxdump tarball into `dest_dir`, rejecting traversal-prone entries.
pub fn extract_taxdump(tarball: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    let tar_gz = File::open(tarball)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Reject absolute paths and parent directory references to prevent
        // directory traversal attacks.
        if path.is_absolute()
            || path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(anyhow::anyhow!(
                "Invalid tar entry path: {}",
                path.display()
            ));
        }

        entry.unpack_in(dest_dir)?;
    }

    Ok(())
}

/// Format human-readable sizes for the downloaded files.
pub fn format_file_sizes(paths: &DownloadPaths) -> anyhow::Result<Vec<String>> {
    let mut sizes = Vec::new();
    for f in &[&paths.tarball, &paths.ar_refseq, &paths.ar_genbank] {
        let size = f.metadata()?.len();
        sizes.push(format!(
            "{}\t{}",
            f.to_string_lossy(),
            readable(&size.to_string())
        ));
    }
    Ok(sizes)
}

/// FTP connection abstraction for testability.
#[cfg_attr(test, mockall::automock)]
pub trait FtpConnectionTrait {
    /// Login to FTP server.
    fn login(&mut self, user: &str, password: &str) -> anyhow::Result<()>;

    /// Change working directory.
    fn cwd(&mut self, path: &str) -> anyhow::Result<()>;

    /// Get current directory.
    fn pwd(&mut self) -> anyhow::Result<String>;

    /// Download a file into memory.
    fn simple_retr(
        &mut self,
        filename: &str,
    ) -> anyhow::Result<std::io::Cursor<Vec<u8>>>;

    /// Download a file to a specific path.
    fn retr_to_file(&mut self, filename: &str, dest_path: &Path) -> anyhow::Result<()>;

    /// Quit connection.
    fn quit(&mut self) -> anyhow::Result<()>;
}

/// Real FTP connection wrapping `ftp::FtpStream`.
pub struct FtpConnection {
    stream: ftp::FtpStream,
}

impl FtpConnection {
    /// Connect to an FTP server.
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
            let copy_result = io::copy(stream, &mut file);
            // Close the file handle before removal so the OS releases the lock
            // (especially on Windows) and the partial file can be deleted.
            drop(file);
            if copy_result.is_err() {
                // Remove the partial file so the next run re-downloads instead
                // of silently reusing a truncated/corrupt download. Assembly
                // reports have no MD5 check, so this cleanup is essential.
                let _ = std::fs::remove_file(dest_path);
            }
            copy_result
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

/// Download the taxdump tarball and its MD5 file from the remote taxonomy path.
pub fn download_taxdump(
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

/// Download RefSeq and GenBank assembly summary reports.
pub fn download_assembly_reports(
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

/// Orchestrate the full download workflow.
pub fn run<F>(
    nwrdir: &Path,
    host: &str,
    tx_path: &str,
    ar_path: &str,
    mut conn_factory: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str) -> anyhow::Result<Box<dyn FtpConnectionTrait>>,
{
    let paths = get_download_paths(nwrdir)?;

    // Download taxdump
    info!("==> Downloading from {} ...", host);
    if taxdump_exists(&paths.tarball) && paths.md5_file.exists() {
        info!("Skipping, {} exists", paths.tarball.to_string_lossy());
    } else {
        info!("Connecting...");
        let mut conn = conn_factory(host)?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        download_taxdump(conn.as_mut(), &paths, tx_path)?;
        conn.quit()?;
        info!("End connection.");
    }

    // Check
    info!("==> Checking...");
    if let Err(e) = check_taxdump_md5(&paths.tarball, &paths.md5_file) {
        // Remove corrupt files so the next run re-downloads instead of
        // reusing the same corrupt tarball (the skip-guard above would
        // otherwise block re-download forever).
        let _ = std::fs::remove_file(&paths.tarball);
        let _ = std::fs::remove_file(&paths.md5_file);
        return Err(e);
    }

    // Extract
    info!("==> Extracting...");
    extract_taxdump(&paths.tarball, nwrdir)?;

    // Assembly reports
    info!("==> Downloading from {} ...", host);
    if assembly_reports_exist(&paths.ar_refseq, &paths.ar_genbank) {
        info!(
            "Skipping, {} & {} exist",
            paths.ar_refseq.to_string_lossy(),
            paths.ar_genbank.to_string_lossy()
        );
    } else {
        info!("Connecting...");
        let mut conn = conn_factory(host)?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        download_assembly_reports(conn.as_mut(), &paths, ar_path)?;
        conn.quit()?;
        info!("End connection.");
    }

    info!("File sizes:");
    for size_line in format_file_sizes(&paths)? {
        info!("{}", size_line);
    }

    Ok(())
}

/// Add thousands separators to a non-negative integer represented as a string.
pub fn readable(n: &str) -> String {
    let mut parts: Vec<char> = Vec::with_capacity(n.len() + n.len() / 3);

    for (i, ch) in n.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            parts.push(',');
        }
        parts.push(ch);
    }

    parts.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

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
        assert_eq!(readable("123"), "123");
    }

    #[test]
    fn test_readable_thousands() {
        assert_eq!(readable("1234"), "1,234");
    }

    #[test]
    fn test_readable_millions() {
        assert_eq!(readable("1234567"), "1,234,567");
    }

    #[test]
    fn test_readable_zero() {
        assert_eq!(readable("0"), "0");
    }

    #[test]
    fn test_readable_large_number() {
        assert_eq!(readable("1234567890"), "1,234,567,890");
    }

    #[test]
    fn test_readable_empty_string() {
        assert_eq!(readable(""), "");
    }

    #[test]
    fn test_readable_exact_boundary() {
        // Test exactly at thousand boundaries
        assert_eq!(readable("1000"), "1,000");
        assert_eq!(readable("1000000"), "1,000,000");
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

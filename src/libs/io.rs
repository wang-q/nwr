use anyhow::Context;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Component, Path};

/// Open a buffered reader for `input`.
///
/// `"stdin"` reads from standard input; a path ending in `.gz` is transparently
/// decompressed via `MultiGzDecoder`. All other paths are opened as plain files.
/// Returns an error instead of panicking when the file cannot be opened.
pub fn reader(input: &str) -> anyhow::Result<Box<dyn BufRead>> {
    if input == "stdin" {
        Ok(Box::new(std::io::BufReader::new(std::io::stdin())))
    } else {
        let path = Path::new(input);
        let file = std::fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("Could not open {}: {}", path.display(), e))?;
        let is_gz = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));
        if is_gz {
            Ok(Box::new(std::io::BufReader::new(
                flate2::read::MultiGzDecoder::new(file),
            )))
        } else {
            Ok(Box::new(std::io::BufReader::new(file)))
        }
    }
}

/// Open a buffered writer for `output`.
///
/// `"stdout"` writes to standard output; any other value is treated as a file
/// path and opened with `File::create`.
pub fn writer(output: &str) -> anyhow::Result<Box<dyn Write>> {
    if output == "stdout" {
        Ok(Box::new(BufWriter::new(std::io::stdout())))
    } else {
        let path = Path::new(output);
        let file = File::create(path)
            .with_context(|| format!("Could not create {}", path.display()))?;
        Ok(Box::new(BufWriter::new(file)))
    }
}

/// Initialize the terminal logger for stderr output.
///
/// Re-initialization errors are ignored so that tests or other callers that
/// already set up a logger do not fail here.
pub fn init_logger() {
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    );
}

/// Validate that a tar entry path does not escape the destination directory.
///
/// Rejects absolute paths and paths containing `..` components.
pub fn validate_tar_entry_path(path: &Path) -> anyhow::Result<()> {
    if path.is_absolute() || path.components().any(|c| c == Component::ParentDir) {
        anyhow::bail!("Invalid tar entry path: {}", path.display());
    }
    Ok(())
}

/// Interval (in iterations) between progress dots printed by [`progress_dot`].
const PROGRESS_INTERVAL: usize = 10000;

/// Print a progress dot every `PROGRESS_INTERVAL` iterations.
///
/// Pass the current 0-based or 1-based loop counter as `i`; the leading dot
/// at `i == 0` is suppressed so callers that enumerate from 0 do not emit a
/// spurious dot on the first iteration. The caller is responsible for printing
/// a trailing newline after the loop completes. Output goes to stderr so it
/// does not pollute stdout when the user redirects data output.
pub fn progress_dot(i: usize) -> anyhow::Result<()> {
    if i > 0 && i.is_multiple_of(PROGRESS_INTERVAL) {
        eprint!(".");
        std::io::stderr().flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tempfile::TempDir;

    #[test]
    fn test_writer_creates_file() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("out.txt");
        {
            let mut w = writer(target.to_str().unwrap()).unwrap();
            write!(w, "hello").unwrap();
        }

        assert!(target.exists());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "hello");
    }

    #[test]
    fn test_writer_overwrites_existing_target() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("out.txt");
        std::fs::write(&target, "old").unwrap();

        {
            let mut w = writer(target.to_str().unwrap()).unwrap();
            write!(w, "new").unwrap();
        }

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
    }

    #[test]
    fn test_validate_tar_entry_path_ok() {
        assert!(validate_tar_entry_path(Path::new("names.dmp")).is_ok());
        assert!(validate_tar_entry_path(Path::new("dir/names.dmp")).is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_tar_entry_path_absolute() {
        assert!(validate_tar_entry_path(Path::new("/etc/passwd")).is_err());
    }

    #[test]
    #[cfg(windows)]
    fn test_validate_tar_entry_path_absolute() {
        assert!(validate_tar_entry_path(Path::new("C:\\Windows\\System32")).is_err());
    }

    #[test]
    fn test_validate_tar_entry_path_parent() {
        assert!(validate_tar_entry_path(Path::new("../names.dmp")).is_err());
        assert!(validate_tar_entry_path(Path::new("dir/../../names.dmp")).is_err());
    }

    #[test]
    fn test_reader_gz_case_insensitive() {
        let dir = TempDir::new().unwrap();

        for ext in ["gz", "GZ", "Gz"] {
            let path = dir.path().join(format!("input.{ext}"));
            {
                let file = std::fs::File::create(&path).unwrap();
                let mut encoder =
                    flate2::write::GzEncoder::new(file, flate2::Compression::default());
                encoder.write_all(b"hello").unwrap();
                encoder.finish().unwrap();
            }

            let mut reader = reader(path.to_str().unwrap()).unwrap();
            let mut buf = String::new();
            reader.read_to_string(&mut buf).unwrap();
            assert_eq!(buf, "hello", "extension .{ext} should be decompressed");
        }
    }
}

use anyhow::Context;
use std::fs::File;
use std::io::{BufRead, BufWriter, Stdout, Write};
use std::path::{Component, Path, PathBuf};

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
            .map(|ext| ext.eq_ignore_ascii_case("gz"))
            .unwrap_or(false);
        if is_gz {
            Ok(Box::new(std::io::BufReader::new(
                flate2::read::MultiGzDecoder::new(file),
            )))
        } else {
            Ok(Box::new(std::io::BufReader::new(file)))
        }
    }
}

/// Atomic writer that commits to the target path only on success.
///
/// For file outputs, data is buffered to a temporary file; calling
/// [`Writer::finish`] renames the temporary file to the target path. If the
/// writer is dropped without calling `finish` (for example, after an error),
/// the temporary file is removed so that partial output is not left behind.
/// `"stdout"` writes directly to standard output.
pub struct Writer {
    inner: WriterInner,
}

enum WriterInner {
    Stdout(BufWriter<Stdout>),
    File {
        temp: PathBuf,
        target: PathBuf,
        file: BufWriter<File>,
        failed: bool,
    },
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            WriterInner::Stdout(w) => w.write(buf),
            WriterInner::File { file, failed, .. } => {
                let res = file.write(buf);
                if res.is_err() {
                    *failed = true;
                }
                res
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            WriterInner::Stdout(w) => w.flush(),
            WriterInner::File { file, .. } => file.flush(),
        }
    }
}

impl Writer {
    /// Flush buffered data and commit the output.
    ///
    /// For file outputs this renames the temporary file to the target path.
    /// Calling `finish` consumes the writer.
    pub fn finish(mut self) -> anyhow::Result<()> {
        match self.inner {
            WriterInner::Stdout(ref mut w) => {
                w.flush()?;
            }
            WriterInner::File {
                ref temp,
                ref target,
                ref mut file,
                failed,
            } => {
                file.flush()
                    .with_context(|| format!("failed to flush {}", temp.display()))?;
                if failed {
                    anyhow::bail!(
                        "writer encountered errors; not committing partial output"
                    );
                }
                std::fs::rename(temp, target).with_context(|| {
                    format!("failed to commit output to {}", target.display())
                })?;
            }
        }
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        match &mut self.inner {
            WriterInner::Stdout(w) => {
                let _ = w.flush();
            }
            WriterInner::File { temp, file, .. } => {
                // Discard any buffered content and remove the uncommitted temp file.
                let _ = file.flush();
                let _ = std::fs::remove_file(temp);
            }
        }
    }
}

/// Open a buffered writer for `output`.
///
/// `"stdout"` writes to standard output; any other value is treated as a file
/// path and is written atomically via a temporary file. The caller must call
/// [`Writer::finish`] on successful completion to commit the output.
pub fn writer(output: &str) -> anyhow::Result<Writer> {
    if output == "stdout" {
        Ok(Writer {
            inner: WriterInner::Stdout(BufWriter::new(std::io::stdout())),
        })
    } else {
        let target = PathBuf::from(output);
        let temp = PathBuf::from(format!("{output}.tmp"));
        let file = File::create(&temp).with_context(|| {
            format!("Could not create temporary file {}", temp.display())
        })?;
        Ok(Writer {
            inner: WriterInner::File {
                temp,
                target,
                file: BufWriter::new(file),
                failed: false,
            },
        })
    }
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
    fn test_writer_atomic_commit() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("out.txt");
        let mut w = writer(target.to_str().unwrap()).unwrap();
        write!(w, "hello").unwrap();
        w.finish().unwrap();

        assert!(target.exists());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "hello");
        assert!(!dir.path().join("out.txt.tmp").exists());
    }

    #[test]
    fn test_writer_drop_without_finish_removes_temp() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("out.txt");
        {
            let mut w = writer(target.to_str().unwrap()).unwrap();
            write!(w, "partial").unwrap();
            // Intentionally not calling finish.
        }

        assert!(!target.exists());
        assert!(!dir.path().join("out.txt.tmp").exists());
    }

    #[test]
    fn test_writer_overwrites_existing_target() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("out.txt");
        std::fs::write(&target, "old").unwrap();

        let mut w = writer(target.to_str().unwrap()).unwrap();
        write!(w, "new").unwrap();
        w.finish().unwrap();

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
    }

    #[test]
    fn test_validate_tar_entry_path_ok() {
        assert!(validate_tar_entry_path(Path::new("names.dmp")).is_ok());
        assert!(validate_tar_entry_path(Path::new("dir/names.dmp")).is_ok());
    }

    #[test]
    fn test_validate_tar_entry_path_absolute() {
        assert!(validate_tar_entry_path(Path::new("/etc/passwd")).is_err());
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

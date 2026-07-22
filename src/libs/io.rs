use std::ffi::OsStr;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

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
        if path.extension() == Some(OsStr::new("gz")) {
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
/// path to create (truncating existing content). Returns an error instead of
/// panicking when the file cannot be created.
pub fn writer(output: &str) -> anyhow::Result<Box<dyn Write>> {
    if output == "stdout" {
        Ok(Box::new(std::io::BufWriter::new(std::io::stdout())))
    } else {
        let file = std::fs::File::create(output)
            .map_err(|e| anyhow::anyhow!("Could not create {}: {}", output, e))?;
        Ok(Box::new(std::io::BufWriter::new(file)))
    }
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

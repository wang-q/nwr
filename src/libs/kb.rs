use flate2::read::GzDecoder;
use std::fs;
use tar::Archive;

/// Parsed options for knowledge-base extraction.
pub struct KbOptions {
    /// Document name to extract (bac120 or ar53).
    pub infile: String,
    /// Output directory.
    pub outdir: String,
}

/// Extract a bundled knowledge-base archive into `outdir`.
///
/// Only the document names `bac120` and `ar53` are supported.
pub fn run(options: &KbOptions) -> anyhow::Result<()> {
    static FILE_BAC: &[u8] = include_bytes!("../../docs/bac120.tar.gz");
    static FILE_AR: &[u8] = include_bytes!("../../docs/ar53.tar.gz");

    let bytes = match options.infile.as_ref() {
        "bac120" => FILE_BAC,
        "ar53" => FILE_AR,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid document name. Valid options: bac120, ar53"
            ))
        }
    };

    fs::create_dir_all(&options.outdir)?;
    let mut archive = Archive::new(GzDecoder::new(bytes));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
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
        entry.unpack_in(&options.outdir)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_kb_invalid_document() {
        let result = run(&KbOptions {
            infile: "invalid".to_string(),
            outdir: ".".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_kb_extract_bac120() {
        let temp_dir = TempDir::new().unwrap();
        let outdir = temp_dir.path().to_str().unwrap().to_string();
        let result = run(&KbOptions {
            infile: "bac120".to_string(),
            outdir,
        });
        assert!(result.is_ok());
        assert!(std::fs::read_dir(temp_dir.path()).unwrap().next().is_some());
    }

    #[test]
    fn test_kb_extract_ar53() {
        let temp_dir = TempDir::new().unwrap();
        let outdir = temp_dir.path().to_str().unwrap().to_string();
        let result = run(&KbOptions {
            infile: "ar53".to_string(),
            outdir,
        });
        assert!(result.is_ok());
        assert!(std::fs::read_dir(temp_dir.path()).unwrap().next().is_some());
    }
}

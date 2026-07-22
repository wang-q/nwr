use log::warn;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;

/// Parsed options for restrict operations.
pub struct RestrictOptions {
    /// Directory containing NCBI taxonomy databases.
    pub nwrdir: PathBuf,
    /// Ancestor terms used to filter input lines.
    pub terms: Vec<String>,
    /// Input TSV files.
    pub files: Vec<String>,
    /// 1-based column index containing the taxon ID.
    pub column: usize,
    /// Whether to exclude (rather than include) matching descendants.
    pub is_exclude: bool,
    /// Output file path.
    pub outfile: String,
}

/// Restrict taxonomy terms to descendants of the given ancestor terms.
///
/// Reads each input file and outputs lines whose taxon ID in the specified
/// column is a descendant of one of the ancestor terms. With `is_exclude`
/// set, outputs lines that are *not* descendants instead.
pub fn run(options: &RestrictOptions) -> anyhow::Result<()> {
    if options.column == 0 {
        return Err(anyhow::anyhow!(
            "Column must be a positive integer (1-based)"
        ));
    }

    let mut writer = crate::libs::io::writer(&options.outfile)?;

    let conn = crate::connect_txdb(&options.nwrdir)?;

    let mut id_set = HashSet::new();
    for term in &options.terms {
        let id = crate::term_to_tax_id(&conn, term)?;
        let descendents = crate::get_all_descendent(&conn, id)?;
        id_set.extend(descendents);
    }

    // Cache term lookups so that input files with duplicate terms don't
    // trigger redundant SQL queries. Failed lookups are also cached so that
    // repeated invalid terms skip without re-querying.
    let mut term_cache: HashMap<String, i64> = HashMap::new();
    let mut term_failed: HashSet<String> = HashSet::new();

    for infile in &options.files {
        let reader = crate::libs::io::reader(infile)?;
        for line in reader.lines() {
            let line = line?;

            // Always output lines start with "#"
            if line.starts_with('#') {
                writer.write_fmt(format_args!("{}\n", line))?;
                continue;
            }

            // Check the given field
            let fields: Vec<&str> = line.split('\t').collect();
            let term = fields.get(options.column - 1).ok_or_else(|| {
                anyhow::anyhow!("Column {} not found in line: {}", options.column, line)
            })?;
            if term_failed.contains(*term) {
                continue;
            }
            let id = match term_cache.get(*term) {
                Some(&id) => id,
                None => match crate::term_to_tax_id(&conn, term) {
                    Ok(x) => {
                        term_cache.insert((*term).to_string(), x);
                        x
                    }
                    Err(err) => {
                        warn!("Error converting term '{}': {}", term, err);
                        term_failed.insert((*term).to_string());
                        continue;
                    }
                },
            };

            if options.is_exclude ^ id_set.contains(&id) {
                writer.write_fmt(format_args!("{}\n", line))?;
            }
        }
    }
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_restrict_include() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        // Create input file
        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#header").unwrap();
        writeln!(file, "phage1\t12347").unwrap(); // Actinophage JHJ-1
        writeln!(file, "phage2\t999999").unwrap(); // Non-virus
        drop(file);

        let result = run(&RestrictOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            files: vec![input_file.to_str().unwrap().to_string()],
            column: 2,
            is_exclude: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#header"));
        assert!(output.contains("12347"));
        assert!(!output.contains("999999"));
    }

    #[test]
    fn test_restrict_exclude() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#header").unwrap();
        writeln!(file, "phage1\t12347").unwrap(); // Actinophage JHJ-1
        writeln!(file, "phage2\t999999").unwrap(); // Non-virus
        drop(file);

        let result = run(&RestrictOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            files: vec![input_file.to_str().unwrap().to_string()],
            column: 2,
            is_exclude: true,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#header"));
        assert!(!output.contains("12347")); // Excluded
        assert!(output.contains("999999")); // Not in Viruses, so included
    }

    #[test]
    fn test_restrict_with_comment_lines() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "# Another comment").unwrap();
        writeln!(file, "data\t12347").unwrap();
        drop(file);

        let result = run(&RestrictOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            files: vec![input_file.to_str().unwrap().to_string()],
            column: 2,
            is_exclude: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Comment lines should be preserved
        assert!(output.contains("# This is a comment"));
        assert!(output.contains("# Another comment"));
    }

    #[test]
    fn test_restrict_multiple_terms() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#header").unwrap();
        writeln!(file, "item1\t12347").unwrap(); // Virus
        writeln!(file, "item2\t10239").unwrap(); // Viruses root
        drop(file);

        let result = run(&RestrictOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string(), "12333".to_string()],
            files: vec![input_file.to_str().unwrap().to_string()],
            column: 2,
            is_exclude: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_restrict_with_zero_column() {
        let result = run(&RestrictOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["10239".to_string()],
            files: vec!["tests/nwr/strains.tsv".to_string()],
            column: 0,
            is_exclude: false,
            outfile: "stdout".to_string(),
        });
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("positive integer"));
    }
}

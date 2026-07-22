use log::warn;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;

/// Parsed options for append operations.
pub struct AppendOptions {
    /// Directory containing NCBI taxonomy databases.
    pub nwrdir: PathBuf,
    /// Input TSV files.
    pub infiles: Vec<String>,
    /// Output file path.
    pub outfile: String,
    /// 1-based column index containing the taxon term or ID.
    pub column: usize,
    /// Taxonomic ranks to append.
    pub ranks: Vec<String>,
    /// Whether the input column contains taxon IDs instead of names.
    pub is_id: bool,
}

/// Append taxonomic rank fields to a TSV file.
///
/// Reads each input file, resolves the term in the specified column to a taxon
/// ID, and appends the requested rank names (and optionally their IDs).
pub fn run(options: &AppendOptions) -> anyhow::Result<()> {
    if options.column == 0 {
        return Err(anyhow::anyhow!(
            "Column must be a positive integer (1-based)"
        ));
    }

    let mut writer = intspan::writer(&options.outfile);

    let conn = crate::connect_txdb(&options.nwrdir)?;

    for infile in &options.infiles {
        let reader = intspan::reader(infile);

        'line: for line in reader.lines() {
            let line = line?;

            // Lines start with "#"
            if line.starts_with('#') {
                let mut fields: Vec<String> =
                    line.split('\t').map(|s| s.to_string()).collect();
                if options.ranks.is_empty() {
                    fields.push("sci_name".to_string());
                    if options.is_id {
                        fields.push("tax_id".to_string());
                    }
                } else {
                    for rank in options.ranks.iter() {
                        fields.push(rank.to_string());
                        if options.is_id {
                            fields.push(format!("{}_id", rank));
                        }
                    }
                }
                let new_line: String = fields.join("\t");
                writer.write_fmt(format_args!("{}\n", new_line))?;
                continue;
            }

            let mut fields: Vec<String> =
                line.split('\t').map(|s| s.to_string()).collect();
            // Normal lines
            // Check the given field
            let term = fields.get(options.column - 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "Column {} out of range (line has {} columns)",
                    options.column,
                    fields.len()
                )
            })?;
            let id = match crate::term_to_tax_id(&conn, term) {
                Ok(x) => x,
                Err(err) => {
                    warn!("Error converting term '{}': {}", term, err);
                    continue 'line;
                }
            };

            if options.ranks.is_empty() {
                let node = match crate::get_taxon(&conn, &[id]) {
                    Ok(x) => x.into_iter().next().ok_or_else(|| {
                        anyhow::anyhow!("No taxon found for ID: {}", id)
                    })?,
                    Err(err) => {
                        warn!("Error getting taxon {}: {}", id, err);
                        continue 'line;
                    }
                };
                let s = node.scientific_name().unwrap_or("Unknown").to_string();

                fields.push(s);
                if options.is_id {
                    fields.push(id.to_string());
                }
            } else {
                let lineage = match crate::get_lineage(&conn, id) {
                    Err(err) => {
                        warn!("Errors on get_lineage({}): {}", id, err);
                        continue 'line;
                    }
                    Ok(x) => x,
                };

                for rank in options.ranks.iter() {
                    let (tax_id, sci_name) = crate::find_rank(&lineage, rank);
                    fields.push(sci_name.to_string());
                    if options.is_id {
                        fields.push(format!("{}", tax_id));
                    }
                }
            }

            let new_line: String = fields.join("\t");
            writer.write_fmt(format_args!("{}\n", new_line))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_append_with_valid_taxon() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        // Create input file with a valid taxon name
        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#header").unwrap();
        writeln!(file, "Actinophage JHJ-1").unwrap();
        drop(file);

        let result = run(&AppendOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            infiles: vec![input_file.to_str().unwrap().to_string()],
            outfile: output_file.to_str().unwrap().to_string(),
            column: 1,
            ranks: vec![],
            is_id: false,
        });
        assert!(result.is_ok());

        // Check output
        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Actinophage JHJ-1"));
        assert!(output.contains("sci_name"));
    }

    #[test]
    fn test_append_with_invalid_taxon() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        // Create input file with an invalid taxon name
        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#header").unwrap();
        writeln!(file, "NonExistentTaxon12345").unwrap();
        drop(file);

        let result = run(&AppendOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            infiles: vec![input_file.to_str().unwrap().to_string()],
            outfile: output_file.to_str().unwrap().to_string(),
            column: 1,
            ranks: vec![],
            is_id: false,
        });
        assert!(result.is_ok()); // Should not error, just skip invalid lines

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#header"));
        // Invalid line should be skipped
        assert!(!output.contains("NonExistentTaxon12345"));
    }

    #[test]
    fn test_append_with_column_option() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "name\tvalue").unwrap();
        writeln!(file, "other\tActinophage JHJ-1").unwrap();
        drop(file);

        let result = run(&AppendOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            infiles: vec![input_file.to_str().unwrap().to_string()],
            outfile: output_file.to_str().unwrap().to_string(),
            column: 2,
            ranks: vec![],
            is_id: false,
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Actinophage JHJ-1"));
    }

    #[test]
    fn test_append_with_rank_option() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#name").unwrap();
        writeln!(file, "Actinophage JHJ-1").unwrap();
        drop(file);

        let result = run(&AppendOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            infiles: vec![input_file.to_str().unwrap().to_string()],
            outfile: output_file.to_str().unwrap().to_string(),
            column: 1,
            ranks: vec!["species".to_string(), "family".to_string()],
            is_id: false,
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("species"));
        assert!(output.contains("family"));
    }

    #[test]
    fn test_append_with_id_option() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.tsv");
        let output_file = temp_dir.path().join("output.tsv");

        let mut file = std::fs::File::create(&input_file).unwrap();
        writeln!(file, "#name").unwrap();
        writeln!(file, "Actinophage JHJ-1").unwrap();
        drop(file);

        let result = run(&AppendOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            infiles: vec![input_file.to_str().unwrap().to_string()],
            outfile: output_file.to_str().unwrap().to_string(),
            column: 1,
            ranks: vec!["species".to_string()],
            is_id: true,
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("species_id"));
    }

    #[test]
    fn test_append_with_zero_column() {
        let result = run(&AppendOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            infiles: vec!["tests/nwr/strains.tsv".to_string()],
            outfile: "stdout".to_string(),
            column: 0,
            ranks: vec![],
            is_id: false,
        });
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("positive integer"));
    }
}

use std::io::Write;

/// Parsed options for info operations.
pub struct InfoOptions {
    /// Directory containing NCBI taxonomy databases.
    pub nwrdir: std::path::PathBuf,
    /// Taxonomy IDs or scientific names to look up.
    pub terms: Vec<String>,
    /// Output results as TSV instead of human-readable text.
    pub is_tsv: bool,
    /// Output file path.
    pub outfile: String,
}

/// Display information for taxonomy IDs or scientific names.
///
/// Resolves each term against the taxonomy database and writes either a
/// human-readable report or a TSV record for each taxon.
pub fn run(options: &InfoOptions) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(&options.outfile)?;

    let conn = crate::connect_txdb(&options.nwrdir)?;

    let mut ids = vec![];
    for term in &options.terms {
        let id = crate::term_to_tax_id(&conn, term)?;
        ids.push(id);
    }

    let nodes = crate::get_taxon(&conn, &ids)?;

    if options.is_tsv {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(writer);

        wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;
        for node in nodes.iter() {
            let sci_name = node.scientific_name().unwrap_or("Unknown");
            wtr.serialize((node.tax_id, sci_name, &node.rank, &node.division))?;
        }
        wtr.flush()?;
    } else {
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            writer.write_fmt(format_args!("{}", node))?;
        }
        writer.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_basic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let outfile = temp_dir.path().join("output.txt");
        let result = run(&InfoOptions {
            nwrdir: std::path::PathBuf::from("tests/nwr/"),
            terms: vec!["12340".to_string()],
            is_tsv: false,
            outfile: outfile.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
        let output = std::fs::read_to_string(&outfile).unwrap();
        assert!(output.contains("Enterobacteria phage 933J"));
    }

    #[test]
    fn test_info_tsv() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let outfile = temp_dir.path().join("output.tsv");
        let result = run(&InfoOptions {
            nwrdir: std::path::PathBuf::from("tests/nwr/"),
            terms: vec!["12340".to_string()],
            is_tsv: true,
            outfile: outfile.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
        let output = std::fs::read_to_string(&outfile).unwrap();
        assert!(output.contains("#tax_id"));
        assert!(output.contains("Enterobacteria phage 933J"));
    }
}

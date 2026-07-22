use std::io::Write;

/// Parsed options for lineage operations.
pub struct LineageOptions {
    /// Directory containing NCBI taxonomy databases.
    pub nwrdir: std::path::PathBuf,
    /// Taxonomy ID or scientific name whose lineage should be printed.
    pub term: String,
    /// Output file path.
    pub outfile: String,
}

/// Output the lineage of a taxonomy term.
///
/// Resolves the term to a taxon ID, fetches the full lineage to the root,
/// and writes one TSV line per ancestor.
pub fn run(options: &LineageOptions) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(&options.outfile)?;

    let conn = crate::connect_txdb(&options.nwrdir)?;

    let id = crate::term_to_tax_id(&conn, &options.term)?;
    let lineage = crate::get_lineage(&conn, id)?;

    for node in lineage.iter() {
        let sci_name = node.scientific_name().unwrap_or("Unknown");
        writer.write_fmt(format_args!(
            "{}\t{}\t{}\n",
            node.rank, sci_name, node.tax_id
        ))?;
    }
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_basic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let outfile = temp_dir.path().join("output.tsv");
        let result = run(&LineageOptions {
            nwrdir: std::path::PathBuf::from("tests/nwr/"),
            term: "12340".to_string(),
            outfile: outfile.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
        let output = std::fs::read_to_string(&outfile).unwrap();
        assert!(output.contains("Enterobacteria phage 933J"));
        assert!(output.contains("species"));
    }
}

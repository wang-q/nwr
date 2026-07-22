use std::collections::HashSet;
use std::path::PathBuf;

/// Parsed options for member operations.
pub struct MemberOptions {
    pub nwrdir: PathBuf,
    pub terms: Vec<String>,
    pub ranks: Vec<String>,
    pub is_env: bool,
    pub outfile: String,
}

/// List taxonomy members under the given ancestor terms.
///
/// Outputs a TSV with tax_id, scientific name, rank and division for each
/// descendant. If `ranks` is non-empty, only those ranks are included. With
/// `is_env` false, members from the "Environmental samples" division are
/// skipped.
pub fn run(options: &MemberOptions) -> anyhow::Result<()> {
    let writer = intspan::writer(&options.outfile);

    let conn = crate::connect_txdb(&options.nwrdir)?;

    let mut tsv_wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    tsv_wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;

    let mut rank_set: HashSet<String> = HashSet::new();
    for rank in &options.ranks {
        rank_set.insert(rank.to_string());
    }

    for term in &options.terms {
        let id = crate::term_to_tax_id(&conn, term)?;
        let descendents = crate::get_all_descendent(&conn, id)?;
        let nodes = crate::get_taxon(&conn, &descendents)?;

        for node in nodes.iter() {
            if !rank_set.is_empty() && !rank_set.contains(&node.rank) {
                continue;
            }
            if !options.is_env && node.division == "Environmental samples" {
                continue;
            }

            let sci_name = node.scientific_name().unwrap_or("Unknown");
            tsv_wtr.serialize((node.tax_id, sci_name, &node.rank, &node.division))?;
        }
    }
    tsv_wtr.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_member_basic() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let result = run(&MemberOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            ranks: vec![],
            is_env: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#tax_id"));
        assert!(output.contains("sci_name"));
    }

    #[test]
    fn test_member_with_rank_filter() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let result = run(&MemberOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            ranks: vec!["species".to_string()],
            is_env: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Should only contain species rank
        for line in output.lines().skip(1) {
            if !line.starts_with('#') {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() >= 3 {
                    assert_eq!(fields[2], "species");
                }
            }
        }
    }

    #[test]
    fn test_member_with_env_flag() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let result = run(&MemberOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            ranks: vec![],
            is_env: true,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_member_without_env_flag() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let result = run(&MemberOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string()],
            ranks: vec![],
            is_env: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Should not contain Environmental samples
        assert!(!output.contains("Environmental samples"));
    }

    #[test]
    fn test_member_with_multiple_terms() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let result = run(&MemberOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["Viruses".to_string(), "10239".to_string()],
            ranks: vec![],
            is_env: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_member_with_tax_id() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let result = run(&MemberOptions {
            nwrdir: PathBuf::from("tests/nwr/"),
            terms: vec!["10239".to_string()],
            ranks: vec![],
            is_env: false,
            outfile: output_file.to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#tax_id"));
        assert!(output.contains("10239"));
    }
}

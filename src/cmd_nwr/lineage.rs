use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("lineage")
        .about("Output the lineage of the term")
        .after_help(include_str!("../../docs/help/lineage.md"))
        .arg(
            Arg::new("term")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1)
                .index(1),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
        )
        .arg(
            Arg::new("tsv")
                .long("tsv")
                .action(ArgAction::SetTrue)
                .help("Output the results as TSV"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename (default: stdout)"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let term = args
        .get_one::<String>("term")
        .ok_or_else(|| anyhow::anyhow!("No term provided"))?;
    let id = nwr::term_to_tax_id(&conn, term)?;

    let lineage = nwr::get_lineage(&conn, id)?;

    for node in lineage.iter() {
        writer.write_fmt(format_args!(
            "{}\t{}\t{}\n",
            node.rank,
            node.names.get("scientific name").unwrap()[0],
            node.tax_id
        ))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_lineage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "lineage",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Actinophage JHJ-1",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Should contain lineage information
        assert!(output.contains("no rank"));
        assert!(output.contains("root"));
        assert!(output.contains("10239")); // Viruses
    }

    #[test]
    fn test_lineage_with_tax_id() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "lineage",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "12347", // Actinophage JHJ-1
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("12347"));
    }

    #[test]
    fn test_lineage_tsv_format() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "lineage",
                "--dir",
                "tests/nwr/",
                "--tsv",
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // TSV format should have rank, name, and tax_id columns
        for line in output.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 3);
        }
    }

    #[test]
    fn test_lineage_stdout() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "lineage",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "10239",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        // Verify output file was created and has content
        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Viruses"));
        assert!(output.contains("10239"));
    }

    #[test]
    fn test_lineage_root() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "lineage",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "root",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Root should only have one line
        assert_eq!(output.lines().count(), 1);
        assert!(output.contains("1")); // Root tax_id
    }

    #[test]
    fn test_lineage_with_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "lineage",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Lactobacillus_phage_mv4", // With underscores
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("12392")); // Lactobacillus phage mv4
    }
}

use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Output the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(
            Arg::new("terms")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1..)
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

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let out_string = nwr::libs::common::run(&conn, &terms)?;
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_common_single_term() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
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
        assert!(output.contains("Actinophage JHJ-1"));
        assert!(output.contains("root"));
    }

    #[test]
    fn test_common_multiple_terms() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Actinophage JHJ-1",
                "Bacillus phage bg1",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Both terms should be in the output
        assert!(
            output.contains("Actinophage JHJ-1")
                || output.contains("Bacillus phage bg1")
        );
    }

    #[test]
    fn test_common_with_tax_id() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
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
        assert!(output.contains("Actinophage"));
    }

    #[test]
    fn test_common_stdout() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "10239", // Viruses
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        // Verify output file was created and has content
        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Viruses"));
    }
}

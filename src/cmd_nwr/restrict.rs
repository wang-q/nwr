use clap::*;
use intspan::IntSpan;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("restrict")
        .about("Restrict taxonomy terms to ancestral descendants")
        .after_help(include_str!("../../docs/help/restrict.md"))
        .arg(
            Arg::new("terms")
                .help("The ancestor(s)")
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
            Arg::new("file")
                .long("file")
                .short('f')
                .num_args(1..)
                .action(ArgAction::Append)
                .default_value("stdin")
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("column")
                .long("column")
                .short('c')
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(usize))
                .help("The column where the IDs are located, starting from 1"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("exclude lines matching terms"),
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

    let column: usize = *args.get_one("column").unwrap();
    let is_exclude = args.get_flag("exclude");

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut id_set = IntSpan::new();
    for term in args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
    {
        let id = nwr::term_to_tax_id(&conn, term)?;
        let descendents: Vec<i32> = nwr::get_all_descendent(&conn, id)?
            .iter()
            .map(|n| *n as i32)
            .collect();
        id_set.add_vec(descendents.as_ref());
    }

    for infile in args
        .get_many::<String>("file")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
    {
        let reader = intspan::reader(infile);
        for line in reader.lines().map_while(Result::ok) {
            // Always output lines start with "#"
            if line.starts_with('#') {
                writer.write_fmt(format_args!("{}\n", line))?;
                continue;
            }

            // Check the given field
            let fields: Vec<&str> = line.split('\t').collect();
            let term = fields.get(column - 1).ok_or_else(|| {
                anyhow::anyhow!("Column {} not found in line: {}", column, line)
            })?;
            let id = nwr::term_to_tax_id(&conn, term)?;

            if is_exclude ^ id_set.contains(id as i32) {
                writer.write_fmt(format_args!("{}\n", line))?;
            }
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "restrict",
                "--dir",
                "tests/nwr/",
                "-c",
                "2",
                "-f",
                input_file.to_str().unwrap(),
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "restrict",
                "--dir",
                "tests/nwr/",
                "-c",
                "2",
                "-f",
                input_file.to_str().unwrap(),
                "-o",
                output_file.to_str().unwrap(),
                "-e",
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#header"));
        assert!(!output.contains("12347")); // Excluded
        assert!(output.contains("999999")); // Not in Viruses, so included
    }

    #[test]
    fn test_restrict_stdin() {
        // This test would require mocking stdin, which is complex
        // For now, just verify the command parses correctly with stdin
        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "restrict",
                "--dir",
                "tests/nwr/",
                "-c",
                "1",
                "Viruses",
            ])
            .unwrap();

        // Verify that the default file is "stdin"
        let files: Vec<&String> = matches.get_many::<String>("file").unwrap().collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "stdin");
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "restrict",
                "--dir",
                "tests/nwr/",
                "-c",
                "2",
                "-f",
                input_file.to_str().unwrap(),
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "restrict",
                "--dir",
                "tests/nwr/",
                "-c",
                "2",
                "-f",
                input_file.to_str().unwrap(),
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
                "12333", // unclassified bacterial viruses
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());
    }
}

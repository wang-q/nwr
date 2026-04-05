use clap::*;
use log::warn;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("append")
        .about("Appends taxonomic rank fields to a TSV file")
        .after_help(include_str!("../../docs/help/append.md"))
        .arg(
            Arg::new("infiles")
                .required(true)
                .num_args(1..)
                .index(1)
                .help("Input TSV file(s) to process. Use 'stdin' for standard input"),
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
            Arg::new("rank")
                .long("rank")
                .short('r')
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Taxonomic rank(s) to append"),
        )
        .arg(
            Arg::new("column")
                .long("column")
                .short('c')
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(usize))
                .help("Column containing taxon IDs/names (1-based)"),
        )
        .arg(
            Arg::new("id")
                .long("id")
                .action(ArgAction::SetTrue)
                .help("Also append taxon IDs for each rank"),
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

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.to_string());
        }
    }
    let is_id = args.get_flag("id");

    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = intspan::reader(infile);

        'line: for line in reader.lines().map_while(Result::ok) {
            let mut fields: Vec<String> =
                line.split('\t').map(|s| s.to_string()).collect();

            // Lines start with "#"
            if line.starts_with('#') {
                if ranks.is_empty() {
                    fields.push("sci_name".to_string());
                    if is_id {
                        fields.push("tax_id".to_string());
                    }
                } else {
                    for rank in ranks.iter() {
                        fields.push(rank.to_string());
                        if is_id {
                            fields.push(format!("{}_id", rank));
                        }
                    }
                }
            }
            // Normal lines
            else {
                // Check the given field
                let term = fields.get(column - 1).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Column {} out of range (line has {} columns)",
                        column,
                        fields.len()
                    )
                })?;
                let id = match nwr::term_to_tax_id(&conn, term) {
                    Ok(x) => x,
                    Err(_) => continue 'line,
                };

                if ranks.is_empty() {
                    let node = nwr::get_taxon(&conn, vec![id])?
                        .into_iter()
                        .next()
                        .ok_or_else(|| {
                            anyhow::anyhow!("No taxon found for ID: {}", id)
                        })?;
                    let s = node
                        .names
                        .get("scientific name")
                        .and_then(|v| v.first())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    fields.push(s);
                    if is_id {
                        fields.push(format!("{}", id));
                    }
                } else {
                    let lineage = match nwr::get_lineage(&conn, id) {
                        Err(err) => {
                            warn!("Errors on get_lineage({}): {}", id, err);
                            continue;
                        }
                        Ok(x) => x,
                    };

                    for rank in ranks.iter() {
                        let (tax_id, sci_name) =
                            nwr::find_rank(&lineage, rank.to_string());
                        fields.push(sci_name.to_string());
                        if is_id {
                            fields.push(format!("{}", tax_id));
                        }
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

        // Create mock args for testing
        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "append",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                input_file.to_str().unwrap(),
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "append",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                input_file.to_str().unwrap(),
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "append",
                "--dir",
                "tests/nwr/",
                "-c",
                "2",
                "-o",
                output_file.to_str().unwrap(),
                input_file.to_str().unwrap(),
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "append",
                "--dir",
                "tests/nwr/",
                "-r",
                "species",
                "-r",
                "family",
                "-o",
                output_file.to_str().unwrap(),
                input_file.to_str().unwrap(),
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "append",
                "--dir",
                "tests/nwr/",
                "-r",
                "species",
                "--id",
                "-o",
                output_file.to_str().unwrap(),
                input_file.to_str().unwrap(),
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("species_id"));
    }

    #[test]
    fn test_append_with_stdin() {
        // Test stdin input
        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from(["append", "--dir", "tests/nwr/", "stdin"])
            .unwrap();

        // This would require mocking stdin, which is complex
        // For now, just verify the command parses correctly
        assert_eq!(matches.get_one::<String>("outfile").unwrap(), "stdout");
    }
}

use clap::*;
use std::collections::HashSet;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("member")
        .about("List members (of certain ranks) under ancestral term(s)")
        .after_help(include_str!("../../docs/help/member.md"))
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
            Arg::new("rank")
                .long("rank")
                .short('r')
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Taxonomic rank(s) to list"),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .action(ArgAction::SetTrue)
                .help("Include division `Environmental samples`"),
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
    let writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut tsv_wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    tsv_wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;

    let mut rank_set: HashSet<String> = HashSet::new();
    if args.contains_id("rank") {
        for rank in args
            .get_many::<String>("rank")
            .ok_or_else(|| anyhow::anyhow!("No ranks provided"))?
        {
            rank_set.insert(rank.to_string());
        }
    }
    let is_env = args.get_flag("env");

    for term in args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
    {
        let id = nwr::term_to_tax_id(&conn, term)?;
        let descendents = nwr::get_all_descendent(&conn, id)?;

        let nodes = nwr::get_taxon(&conn, descendents)?;

        for node in nodes.iter() {
            if !rank_set.is_empty() && !rank_set.contains(&node.rank) {
                continue;
            }
            if !is_env && node.division == "Environmental samples" {
                continue;
            }

            let sci_name = node
                .names
                .get("scientific name")
                .and_then(|v| v.first())
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("#tax_id"));
        assert!(output.contains("sci_name"));
    }

    #[test]
    fn test_member_with_rank_filter() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
                "--dir",
                "tests/nwr/",
                "-r",
                "species",
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
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

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
                "--dir",
                "tests/nwr/",
                "--env",
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());
    }

    #[test]
    fn test_member_without_env_flag() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Should not contain Environmental samples
        assert!(!output.contains("Environmental samples"));
    }

    #[test]
    fn test_member_with_multiple_terms() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Viruses",
                "10239", // Viruses tax_id
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());
    }

    #[test]
    fn test_member_with_tax_id() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "10239", // Viruses tax_id
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());
    }

    #[test]
    fn test_member_stdout() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.tsv");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "member",
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
        assert!(output.contains("#tax_id"));
        assert!(output.contains("10239"));
    }
}

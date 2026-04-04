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

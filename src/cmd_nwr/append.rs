use clap::*;
use log::warn;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("append")
        .about("Append fields of higher ranks to a TSV file")
        .after_help(
            r###"
* If `--rank` is empty, the scientific name will be appended.

* Valid ranks
  * species genus family order class phylum kingdom
  * Use other ranks, such as clade or no rank, at your own risk.

* If the desired rank does not present, `NA` will be appended.

* Lines starting with "#" will be treated as headers and have ranks attached to them.

"###,
        )
        .arg(
            Arg::new("infiles")
                .required(true)
                .num_args(1..)
                .index(1)
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Change working directory"),
        )
        .arg(
            Arg::new("rank")
                .long("rank")
                .short('r')
                .num_args(1..)
                .action(ArgAction::Append)
                .help("To list which rank(s)"),
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
            Arg::new("id")
                .long("id")
                .action(ArgAction::SetTrue)
                .help("Also append rank id"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let column: usize = *args.get_one("column").unwrap();

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };

    let conn = nwr::connect_txdb(&nwrdir).unwrap();

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.to_string());
        }
    }
    let is_id = args.get_flag("id");

    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = intspan::reader(infile);

        for line in reader.lines().map_while(Result::ok) {
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
                let term = fields.get(column - 1).unwrap();
                let id = nwr::term_to_tax_id(&conn, term).unwrap();

                if ranks.is_empty() {
                    let node = nwr::get_taxon(&conn, vec![id])
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .clone();
                    let s = &node.names.get("scientific name").unwrap()[0];

                    fields.push(s.to_string());
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

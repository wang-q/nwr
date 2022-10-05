use clap::*;
use std::collections::HashSet;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("member")
        .about("List members (of certain ranks) under ancestral term(s)")
        .after_help(
            r###"
* Ancestral terms are in the form of a Taxonomy ID or scientific name.

* Valid ranks
  * species genus family order class phylum kingdom
  * Use other ranks, such as clade or no rank, at your own risk.

* The output file is in the same TSV format as `nwr info --tsv`.

"###,
        )
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
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };

    let conn = nwr::connect_txdb(&nwrdir).unwrap();

    let mut tsv_wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    tsv_wtr.write_record(&["#tax_id", "sci_name", "rank", "division"])?;

    let mut rank_set: HashSet<String> = HashSet::new();
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            rank_set.insert(rank.to_string());
        }
    }
    let is_env = args.get_flag("env");

    for term in args.get_many::<String>("terms").unwrap() {
        let id = nwr::term_to_tax_id(&conn, term.to_string()).unwrap();
        let descendents = nwr::get_all_descendent(&conn, id).unwrap();

        let nodes = nwr::get_node(&conn, descendents)?;

        for node in nodes.iter() {
            if !rank_set.is_empty() && !rank_set.contains(&node.rank) {
                continue;
            }
            if !is_env && node.division == "Environmental samples" {
                continue;
            }

            tsv_wtr.serialize((
                node.tax_id,
                &node.names.get("scientific name").unwrap()[0],
                &node.rank,
                &node.division,
            ))?;
        }
    }
    tsv_wtr.flush()?;

    Ok(())
}

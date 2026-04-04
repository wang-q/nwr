use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("info")
        .about("Information of Taxonomy ID(s) or scientific name(s)")
        .after_help(include_str!("../../docs/help/info.md"))
        .arg(
            Arg::new("terms")
                .help("Taxonomy ID(s) or scientific name(s)")
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

    let mut ids = vec![];
    for term in args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
    {
        let id = nwr::term_to_tax_id(&conn, term)?;
        ids.push(id);
    }

    let nodes = nwr::get_taxon(&conn, ids)?;

    if args.get_flag("tsv") {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(writer);

        wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;
        for node in nodes.iter() {
            let sci_name = node
                .names
                .get("scientific name")
                .and_then(|v| v.first())
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
            wtr.serialize((node.tax_id, sci_name, &node.rank, &node.division))?;
        }
        wtr.flush()?;
    } else {
        for node in nodes.iter() {
            writer.write_fmt(format_args!("{}", node))?;
        }
    }

    Ok(())
}

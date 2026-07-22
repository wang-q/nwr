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
        let sci_name = node.scientific_name().unwrap_or("Unknown");
        writer.write_fmt(format_args!(
            "{}\t{}\t{}\n",
            node.rank, sci_name, node.tax_id
        ))?;
    }

    Ok(())
}

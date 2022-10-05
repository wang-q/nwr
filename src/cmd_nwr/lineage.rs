use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("lineage")
        .about("Output the lineage of the term")
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
                .help("Change working directory"),
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
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };

    let conn = nwr::connect_txdb(&nwrdir).unwrap();

    let term = args.get_one::<String>("term").unwrap();
    let id = nwr::term_to_tax_id(&conn, term.to_string()).unwrap();

    let lineage = nwr::get_lineage(&conn, id).unwrap();

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

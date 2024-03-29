use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("info")
        .about("Information of Taxonomy ID(s) or scientific name(s)")
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
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };

    let conn = nwr::connect_txdb(&nwrdir).unwrap();

    let mut ids = vec![];
    for term in args.get_many::<String>("terms").unwrap() {
        let id = nwr::term_to_tax_id(&conn, term).unwrap();
        ids.push(id);
    }

    let nodes = nwr::get_taxon(&conn, ids).unwrap();

    if args.get_flag("tsv") {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(writer);

        wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;
        for node in nodes.iter() {
            wtr.serialize((
                node.tax_id,
                &node.names.get("scientific name").unwrap()[0],
                &node.rank,
                &node.division,
            ))?;
        }
        wtr.flush()?;
    } else {
        for node in nodes.iter() {
            writer.write_fmt(format_args!("{}", node))?;
        }
    }

    Ok(())
}

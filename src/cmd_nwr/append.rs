use clap::*;

/// Create clap subcommand arguments.
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

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let column: usize = *args.get_one("column").unwrap();

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.to_string());
        }
    }

    let infiles: Vec<String> = args
        .get_many::<String>("infiles")
        .unwrap()
        .cloned()
        .collect();

    nwr::libs::taxonomy::append::run(&nwr::libs::taxonomy::append::AppendOptions {
        nwrdir,
        infiles,
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
        column,
        ranks,
        is_id: args.get_flag("id"),
    })
}

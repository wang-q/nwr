use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("restrict")
        .about("Restrict taxonomy terms to ancestral descendants")
        .after_help(include_str!("../../docs/help/restrict.md"))
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
            Arg::new("file")
                .long("file")
                .short('f')
                .num_args(1..)
                .action(ArgAction::Append)
                .default_value("stdin")
                .help("Input filename. [stdin] for standard input"),
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
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("exclude lines matching terms"),
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
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let column: usize = *args.get_one("column").unwrap();
    let is_exclude = args.get_flag("exclude");

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let files: Vec<String> = args
        .get_many::<String>("file")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    nwr::libs::taxonomy::restrict::run(&nwr::libs::taxonomy::restrict::RestrictOptions {
        nwrdir,
        terms,
        files,
        column,
        is_exclude,
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

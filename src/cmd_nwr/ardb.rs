use clap::*;
use simplelog::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("ardb")
        .about("Init the assembly database")
        .after_help(include_str!("../../docs/help/ardb.md"))
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
        )
        .arg(
            Arg::new("genbank")
                .long("genbank")
                .action(ArgAction::SetTrue)
                .help("Create the GenBank assembly database"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Debug, Config::default())?;

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    let tx_conn = nwr::connect_txdb(&nwrdir)?;
    nwr::libs::ardb::run(&nwrdir, args.get_flag("genbank"), &tx_conn)
}

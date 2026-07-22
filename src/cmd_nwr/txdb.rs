use clap::*;
use simplelog::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("txdb")
        .about("Init the taxonomy database")
        .after_help(include_str!("../../docs/help/txdb.md"))
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Debug, Config::default())?;

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    nwr::libs::db::txdb::run(&nwrdir)
}

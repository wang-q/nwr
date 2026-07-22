use super::args;
use clap::*;
use simplelog::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("txdb")
        .about("Initializes the taxonomy database")
        .after_help(include_str!("../../docs/help/txdb.md"))
        .arg(args::dir_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    nwr::libs::db::txdb::run(&nwrdir)
}

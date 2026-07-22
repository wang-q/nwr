use clap::*;
use simplelog::*;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("download")
        .about("Downloads the latest releases of `taxdump` and assembly reports")
        .after_help(include_str!("../../docs/help/download.md"))
        .arg(
            Arg::new("host")
                .long("host")
                .num_args(1)
                .default_value("ftp.ncbi.nih.gov:21")
                .help("NCBI FTP Host:Port"),
        )
        .arg(
            Arg::new("tx")
                .long("tx")
                .num_args(1)
                .default_value("/pub/taxonomy")
                .help("NCBI FTP Path of taxonomy"),
        )
        .arg(
            Arg::new("ar")
                .long("ar")
                .num_args(1)
                .default_value("/genomes/ASSEMBLY_REPORTS")
                .help("NCBI FTP Path of assembly reports"),
        )
}

/// Command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let nwrdir = nwr::nwr_path()?;
    let host = args.get_one::<String>("host").unwrap();
    let tx_path = args.get_one::<String>("tx").unwrap();
    let ar_path = args.get_one::<String>("ar").unwrap();

    nwr::libs::db::download::run(&nwrdir, host, tx_path, ar_path, |host| {
        let conn = nwr::libs::db::download::FtpConnection::connect(host)?;
        Ok(Box::new(conn) as Box<dyn nwr::libs::db::download::FtpConnectionTrait>)
    })
}

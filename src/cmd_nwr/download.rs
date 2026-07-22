use clap::*;
use simplelog::*;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("download")
        .about("Download the latest releases of `taxdump` and assembly reports")
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

    nwr::libs::download::run(&nwrdir, host, tx_path, ar_path, |host| {
        let conn = nwr::libs::download::FtpConnection::connect(host)?;
        Ok(Box::new(conn) as Box<dyn nwr::libs::download::FtpConnectionTrait>)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_subcommand() {
        let cmd = make_subcommand();
        assert_eq!(cmd.get_name(), "download");

        // Verify all arguments exist
        let arg_names: Vec<_> =
            cmd.get_arguments().map(|a| a.get_id().as_str()).collect();
        assert!(arg_names.contains(&"host"));
        assert!(arg_names.contains(&"tx"));
        assert!(arg_names.contains(&"ar"));
    }

    #[test]
    fn test_make_subcommand_default_values() {
        let cmd = make_subcommand();

        let host_arg = cmd.get_arguments().find(|a| a.get_id() == "host").unwrap();
        assert_eq!(host_arg.get_default_values(), vec!["ftp.ncbi.nih.gov:21"]);

        let tx_arg = cmd.get_arguments().find(|a| a.get_id() == "tx").unwrap();
        assert_eq!(tx_arg.get_default_values(), vec!["/pub/taxonomy"]);

        let ar_arg = cmd.get_arguments().find(|a| a.get_id() == "ar").unwrap();
        assert_eq!(
            ar_arg.get_default_values(),
            vec!["/genomes/ASSEMBLY_REPORTS"]
        );
    }

    #[test]
    fn test_make_subcommand_about() {
        let cmd = make_subcommand();
        let about = cmd.get_about().map(|s| s.to_string());
        assert!(about.is_some());
        assert!(about.unwrap().to_lowercase().contains("download"));
    }
}

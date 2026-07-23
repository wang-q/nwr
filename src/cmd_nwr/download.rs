use super::args;
use clap::*;
use log::info;
use simplelog::*;

use nwr::libs::download::{
    assembly_reports_exist, check_taxdump_md5, download_assembly_reports,
    download_taxdump, extract_taxdump, format_file_sizes, get_download_paths,
    taxdump_exists, FtpConnection, FtpConnectionTrait,
};

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("download")
        .about("Downloads the latest releases of `taxdump` and assembly reports")
        .after_help(include_str!("../../docs/help/download.md"))
        .arg(args::dir_arg())
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

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    let host = args.get_one::<String>("host").unwrap();
    let tx_path = args.get_one::<String>("tx").unwrap();
    let ar_path = args.get_one::<String>("ar").unwrap();

    let paths = get_download_paths(&nwrdir)?;

    // Download taxdump
    info!("==> Downloading from {} ...", host);
    if taxdump_exists(&paths.tarball) && paths.md5_file.exists() {
        info!("Skipping, {} exists", paths.tarball.to_string_lossy());
    } else {
        info!("Connecting...");
        let mut conn = FtpConnection::connect(host)?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        download_taxdump(&mut conn, &paths, tx_path)?;
        conn.quit()?;
        info!("End connection.");
    }

    // Check
    info!("==> Checking...");
    if let Err(e) = check_taxdump_md5(&paths.tarball, &paths.md5_file) {
        // Remove corrupt files so the next run re-downloads instead of
        // reusing the same corrupt tarball (the skip-guard above would
        // otherwise block re-download forever).
        let _ = std::fs::remove_file(&paths.tarball);
        let _ = std::fs::remove_file(&paths.md5_file);
        return Err(e);
    }

    // Extract
    info!("==> Extracting...");
    extract_taxdump(&paths.tarball, &nwrdir)?;

    // Assembly reports
    info!("==> Downloading from {} ...", host);
    if assembly_reports_exist(&paths.ar_refseq, &paths.ar_genbank) {
        info!(
            "Skipping, {} & {} exist",
            paths.ar_refseq.to_string_lossy(),
            paths.ar_genbank.to_string_lossy()
        );
    } else {
        info!("Connecting...");
        let mut conn = FtpConnection::connect(host)?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        download_assembly_reports(&mut conn, &paths, ar_path)?;
        conn.quit()?;
        info!("End connection.");
    }

    info!("File sizes:");
    for size_line in format_file_sizes(&paths)? {
        info!("{}", size_line);
    }

    Ok(())
}

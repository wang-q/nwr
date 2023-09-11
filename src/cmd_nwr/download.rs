use clap::*;
use log::{info, warn};
use simplelog::*;
use std::fs::File;
use std::io;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("download")
        .about("Download the latest releases of `taxdump` and assembly reports")
        .after_help(
            r###"
You can download the files manually.

mkdir -p ~/.nwr

# taxdump
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz.md5

# assembly reports
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_refseq.txt
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_genbank.txt

# with aria2
cat <<EOF > download.txt
https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz
https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz.md5
https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_refseq.txt
https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_genbank.txt

EOF

aria2c -x 4 -s 2 -c -d ~/.nwr -i download.txt

"###,
        )
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

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    let nwrdir = nwr::nwr_path();
    let tarball = nwrdir.join("taxdump.tar.gz");
    let ar_refseq = nwrdir.join("assembly_summary_refseq.txt");
    let ar_genbank = nwrdir.join("assembly_summary_genbank.txt");

    // download taxdump
    info!(
        "==> Downloading from {} ...",
        args.get_one::<String>("host").unwrap()
    );
    if std::path::Path::new(&tarball).exists() {
        info!("Skipping, {} exists", tarball.to_string_lossy());
    } else {
        info!("Connecting...");
        let mut conn = ftp::FtpStream::connect(args.get_one::<String>("host").unwrap())?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        conn.cwd(args.get_one::<String>("tx").unwrap())?;
        info!("Remote directory: {}", conn.pwd().unwrap());

        info!("Retrieving MD5 file...");
        let mut file = File::create(nwrdir.join("taxdump.tar.gz.md5"))?;
        let mut cursor = conn.simple_retr("taxdump.tar.gz.md5")?;
        io::copy(&mut cursor, &mut file)?;

        info!("Retrieving {}...", "taxdump.tar.gz");
        conn.retr("taxdump.tar.gz", |stream| {
            let mut file = match File::create(&tarball) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file).map_err(ftp::FtpError::ConnectionError)
        })?;

        conn.quit()?;
        info!("End connection.");
    }

    // check
    info!("==> Checking...");
    {
        let mut file = File::open(&tarball)?;
        let mut hasher = md5::Context::new();
        info!("Computing MD5 sum...");
        io::copy(&mut file, &mut hasher)?;
        let digest = format!("{:x}", hasher.compute());

        let mut ncbi_digest =
            std::fs::read_to_string(nwrdir.join("taxdump.tar.gz.md5"))?;
        ncbi_digest.truncate(32);

        if digest != ncbi_digest {
            warn!("Expected sum is: {}", ncbi_digest);
            warn!("Computed sum is: {}", digest);
            panic!("Fail to check integrity.");
        } else {
            info!("MD5 sum passed");
        }
    }

    // extract
    info!("==> Extracting...");
    {
        let tar_gz = File::open(&tarball)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);

        let mut archive = tar::Archive::new(tar);
        archive.unpack(nwrdir)?;
    }

    // assembly reports
    info!(
        "==> Downloading from {} ...",
        args.get_one::<String>("host").unwrap()
    );
    if std::path::Path::new(&ar_refseq).exists()
        && std::path::Path::new(&ar_genbank).exists()
    {
        info!(
            "Skipping, {} & {} exist",
            ar_refseq.to_string_lossy(),
            ar_genbank.to_string_lossy()
        );
    } else {
        info!("Connecting...");
        let mut conn = ftp::FtpStream::connect(args.get_one::<String>("host").unwrap())?;
        conn.login("ftp", "example@example.com")?;
        info!("Connected.");
        conn.cwd(args.get_one::<String>("ar").unwrap())?;
        info!("Remote directory: {}", conn.pwd().unwrap());

        info!("Retrieving {}...", "assembly_summary_refseq.txt");
        conn.retr("assembly_summary_refseq.txt", |stream| {
            let mut file = match File::create(&ar_refseq) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file).map_err(ftp::FtpError::ConnectionError)
        })?;

        info!("Retrieving {}...", "assembly_summary_genbank.txt");
        conn.retr("assembly_summary_genbank.txt", |stream| {
            let mut file = match File::create(&ar_genbank) {
                Err(e) => return Err(ftp::FtpError::ConnectionError(e)),
                Ok(f) => f,
            };
            io::copy(stream, &mut file).map_err(ftp::FtpError::ConnectionError)
        })?;

        conn.quit()?;
        info!("End connection.");
    }

    info!("File sizes:");
    for f in &[tarball, ar_refseq, ar_genbank] {
        info!(
            "{}\t{}",
            f.to_string_lossy(),
            readable(f.metadata().unwrap().len().to_string())
        );
    }

    Ok(())
}

fn readable(n: String) -> String {
    let mut c = String::new();

    for (i, char) in n.chars().rev().enumerate() {
        if i % 3 == 0 && i != 0 {
            c.insert(0, ',');
        }
        c.insert(0, char);
    }

    c
}

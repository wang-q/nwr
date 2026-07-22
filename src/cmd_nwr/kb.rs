use clap::*;
use flate2::read::GzDecoder;
use std::fs;
use tar::Archive;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("kb")
        .about("Prints docs (knowledge bases)")
        .after_help(include_str!("../../docs/help/kb.md"))
        .arg(
            Arg::new("infile")
                .help("Document to print (bac120, ar53)")
                .num_args(1)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("outdir")
                .short('o')
                .long("outdir")
                .num_args(1)
                .default_value(".")
                .help("Output directory (default: current directory)"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outdir = args.get_one::<String>("outdir").unwrap();

    static FILE_BAC: &[u8] = include_bytes!("../../docs/bac120.tar.gz");
    static FILE_AR: &[u8] = include_bytes!("../../docs/ar53.tar.gz");

    match args.get_one::<String>("infile").unwrap().as_ref() {
        "bac120" => {
            fs::create_dir_all(outdir)?;
            let mut archive = Archive::new(GzDecoder::new(FILE_BAC));
            archive.unpack(outdir)?;
        }
        "ar53" => {
            fs::create_dir_all(outdir)?;
            let mut archive = Archive::new(GzDecoder::new(FILE_AR));
            archive.unpack(outdir)?;
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid document name. Valid options: bac120, ar53"
            ))
        }
    };

    Ok(())
}

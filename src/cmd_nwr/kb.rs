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
    let outfile = args.get_one::<String>("outfile").unwrap();

    static FILE_BAC: &[u8] = include_bytes!("../../docs/bac120.tar.gz");
    static FILE_AR: &[u8] = include_bytes!("../../docs/ar53.tar.gz");

    match args.get_one::<String>("infile").unwrap().as_ref() {
        "bac120" => {
            fs::create_dir_all(outfile)?;
            let mut archive = Archive::new(GzDecoder::new(FILE_BAC));
            archive.unpack(outfile)?;
        }
        "ar53" => {
            fs::create_dir_all(outfile)?;
            let mut archive = Archive::new(GzDecoder::new(FILE_AR));
            archive.unpack(outfile)?;
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid document name. Valid options: bac120, ar53"
            ))
        }
    };

    Ok(())
}

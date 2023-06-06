use clap::*;
use flate2::read::GzDecoder;
use std::fs;
use tar::Archive;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("kb")
        .about("Prints docs (knowledge bases)")
        .after_help(
            r###"
* formats - File formats

* bac120  - 120 bacterial protein families
* fungi61 - 61 fungal protein families

"###,
        )
        .arg(
            Arg::new("infile")
                .help("Sets the input file to use")
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
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outfile = args.get_one::<String>("outfile").unwrap();

    static FILE_FORMATS: &str = include_str!("../../doc/formats.md");
    static FILE_BAC: &[u8] = include_bytes!("../../doc/bac120.tar.gz");
    static FILE_FUNGI: &[u8] = include_bytes!("../../doc/fungi61.tar.gz");

    match args.get_one::<String>("infile").unwrap().as_ref() {
        "formats" => {
            let mut writer = intspan::writer(outfile);
            writer.write_all(FILE_FORMATS.as_ref())?;
        }
        "fungi61" => {
            fs::create_dir_all(outfile)?;
            let mut archive = Archive::new(GzDecoder::new(FILE_FUNGI));
            archive.unpack(outfile)?;
        }
        _ => unreachable!(),
    };

    Ok(())
}

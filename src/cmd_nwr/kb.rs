use super::args;
use clap::{Arg, ArgMatches, Command};
use flate2::read::GzDecoder;
use std::fs;
use tar::Archive;

/// Create clap subcommand arguments.
#[must_use]
pub fn make_subcommand() -> Command {
    Command::new("kb")
        .about("Extracts bundled knowledge-base archives")
        .after_help(include_str!("../../docs/help/kb.md"))
        .arg(
            Arg::new("infile")
                .help("Document to print (bac120, ar53)")
                .num_args(1)
                .required(true)
                .index(1),
        )
        .arg(args::outdir_arg())
}

static FILE_BAC: &[u8] = include_bytes!("../../docs/bac120.tar.gz");
static FILE_AR: &[u8] = include_bytes!("../../docs/ar53.tar.gz");

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args
        .get_one::<String>("infile")
        .ok_or_else(|| anyhow::anyhow!("Missing 'infile' argument"))?;
    let outdir = args
        .get_one::<String>("outdir")
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' argument"))?;

    let bytes = match infile.as_str() {
        "bac120" => FILE_BAC,
        "ar53" => FILE_AR,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid document name. Valid options: bac120, ar53"
            ))
        }
    };

    fs::create_dir_all(outdir)?;
    let mut archive = Archive::new(GzDecoder::new(bytes));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.is_absolute()
            || path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(anyhow::anyhow!(
                "Invalid tar entry path: {}",
                path.display()
            ));
        }
        entry.unpack_in(outdir)?;
    }

    Ok(())
}

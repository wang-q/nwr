use super::args;
use clap::*;
use std::io::Write;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("lineage")
        .about("Outputs the lineage of the term")
        .after_help(include_str!("../../docs/help/lineage.md"))
        .arg(
            Arg::new("term")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1)
                .index(1),
        )
        .arg(args::dir_arg())
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    let term = args
        .get_one::<String>("term")
        .ok_or_else(|| anyhow::anyhow!("No term provided"))?;

    let mut writer = nwr::libs::io::writer(
        args.get_one::<String>("outfile")
            .ok_or_else(|| anyhow::anyhow!("Missing 'outfile' argument"))?,
    )?;
    let conn = nwr::connect_txdb(&nwrdir)?;

    let id = nwr::term_to_tax_id(&conn, term)?;
    let lineage = nwr::get_lineage(&conn, id)?;

    for node in lineage.iter() {
        let sci_name = node.scientific_name().unwrap_or("Unknown");
        writer.write_fmt(format_args!(
            "{}\t{}\t{}\n",
            node.rank, sci_name, node.tax_id
        ))?;
    }
    writer.flush()?;

    Ok(())
}

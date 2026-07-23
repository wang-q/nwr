use super::args;
use clap::*;
use std::io::Write;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("info")
        .about("Shows information of Taxonomy ID(s) or scientific name(s)")
        .after_help(include_str!("../../docs/help/info.md"))
        .arg(args::terms_arg("Taxonomy ID(s) or scientific name(s)"))
        .arg(args::dir_arg())
        .arg(
            Arg::new("tsv")
                .long("tsv")
                .action(ArgAction::SetTrue)
                .help("Output the results as TSV"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();
    let is_tsv = args.get_flag("tsv");

    let mut writer = nwr::libs::io::writer(
        args.get_one::<String>("outfile")
            .ok_or_else(|| anyhow::anyhow!("Missing 'outfile' argument"))?,
    )?;
    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut ids = vec![];
    for term in &terms {
        let id = nwr::term_to_tax_id(&conn, term)?;
        ids.push(id);
    }

    let nodes = nwr::get_taxon(&conn, &ids)?;

    if is_tsv {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(writer);

        wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;
        for node in nodes.iter() {
            let sci_name = node.scientific_name().unwrap_or("Unknown");
            wtr.serialize((node.tax_id, sci_name, &node.rank, &node.division))?;
        }
        wtr.flush()?;
    } else {
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            writer.write_fmt(format_args!("{}", node))?;
        }
        writer.flush()?;
    }

    Ok(())
}

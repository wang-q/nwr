use super::args;
use clap::*;
use std::collections::HashSet;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("member")
        .about("Lists members (of certain ranks) under ancestral term(s)")
        .after_help(include_str!("../../docs/help/member.md"))
        .arg(args::terms_arg("The ancestor(s)"))
        .arg(args::dir_arg())
        .arg(args::rank_arg())
        .arg(
            Arg::new("env")
                .long("env")
                .action(ArgAction::SetTrue)
                .help("Include division `Environmental samples`"),
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

    let ranks: Vec<String> = args
        .get_many::<String>("rank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let is_env = args.get_flag("env");

    let writer = nwr::libs::io::writer(
        args.get_one::<String>("outfile")
            .ok_or_else(|| anyhow::anyhow!("Missing 'outfile' argument"))?,
    )?;
    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut tsv_wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(writer);
    tsv_wtr.write_record(["#tax_id", "sci_name", "rank", "division"])?;

    let mut rank_set: HashSet<String> = HashSet::new();
    for rank in &ranks {
        rank_set.insert(rank.to_string());
    }

    // Track seen tax_ids so that overlapping ancestor terms (e.g. "Viruses"
    // and its tax_id 10239) do not produce duplicate rows in the output.
    let mut seen: HashSet<i64> = HashSet::new();

    for term in &terms {
        let id = nwr::term_to_tax_id(&conn, term)?;
        let descendents = nwr::get_all_descendent(&conn, id)?;
        let nodes = nwr::get_taxon(&conn, &descendents)?;

        for node in nodes.iter() {
            if !seen.insert(node.tax_id) {
                continue;
            }
            if !rank_set.is_empty() && !rank_set.contains(&node.rank) {
                continue;
            }
            if !is_env && node.division == "Environmental samples" {
                continue;
            }

            let sci_name = node.scientific_name().unwrap_or("Unknown");
            tsv_wtr.serialize((node.tax_id, sci_name, &node.rank, &node.division))?;
        }
    }
    tsv_wtr.flush()?;

    Ok(())
}

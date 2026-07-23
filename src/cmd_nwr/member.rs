use super::args;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashSet;

/// Create clap subcommand arguments.
#[must_use]
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

    let rank_set: HashSet<String> = ranks.into_iter().collect();

    let ancestor_ids = nwr::terms_to_tax_ids(&conn, &terms)?;

    // Collect all descendant IDs from every ancestor, deduplicating across
    // overlapping subtrees so each taxon is fetched from the database once.
    let mut all_descendants: Vec<i64> = Vec::new();
    let mut seen_descendants: HashSet<i64> = HashSet::new();
    for id in ancestor_ids {
        if !seen_descendants.insert(id) {
            continue;
        }
        all_descendants.push(id);
        let descendants = nwr::get_all_descendent(&conn, id)?;
        for d in descendants {
            // `get_all_descendent` includes the ancestor ID itself; skip it
            // because it was already added above.
            if d == id {
                continue;
            }
            if seen_descendants.insert(d) {
                all_descendants.push(d);
            }
        }
    }

    let nodes = nwr::get_taxon(&conn, &all_descendants)?;

    // Track seen tax_ids so that overlapping ancestor terms (e.g. "Viruses"
    // and its tax_id 10239) do not produce duplicate rows in the output.
    let mut seen: HashSet<i64> = HashSet::new();
    for node in &nodes {
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
    tsv_wtr.flush()?;
    let writer = tsv_wtr
        .into_inner()
        .map_err(|e| anyhow::anyhow!("failed to flush TSV writer: {e}"))?;
    writer.finish()?;

    Ok(())
}

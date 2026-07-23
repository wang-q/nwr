use super::args;
use clap::*;
use std::collections::HashMap;
use std::io::Write;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Outputs the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(args::terms_arg("The NCBI Taxonomy ID or scientific name"))
        .arg(args::dir_arg())
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

    let mut writer = nwr::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;
    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut tax_ids = vec![];
    for term in &terms {
        let id = nwr::term_to_tax_id(&conn, term)?;
        tax_ids.push(id);
    }

    let mut tree = phylotree::tree::Tree::new();
    // tax_id to NodeId
    let mut id_of: HashMap<i64, usize> = HashMap::new();

    for tax_id in tax_ids {
        let lineage = nwr::get_lineage(&conn, tax_id)?;

        for taxon in lineage.iter() {
            let cur_tax_id = taxon.tax_id;
            if !id_of.contains_key(&cur_tax_id) {
                let node_id = if cur_tax_id == 1 {
                    nwr::libs::taxonomy::common::add_taxon(&mut tree, taxon, None)?
                } else {
                    let parent_tax_id = taxon.parent_tax_id;
                    let parent_id = id_of.get(&parent_tax_id).ok_or_else(|| {
                        anyhow::anyhow!("Parent ID not found: {}", parent_tax_id)
                    })?;
                    nwr::libs::taxonomy::common::add_taxon(
                        &mut tree,
                        taxon,
                        Some(*parent_id),
                    )?
                };
                id_of.insert(cur_tax_id, node_id);
            }
        }
    }

    tree.compress()?;
    let out_string = tree.to_newick()?;
    writeln!(writer, "{}", out_string)?;
    writer.flush()?;

    Ok(())
}

use clap::*;
use nwr::Taxon;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Output the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(
            Arg::new("terms")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
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
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut tax_ids = vec![];
    for term in args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
    {
        let id = nwr::term_to_tax_id(&conn, term)?;
        tax_ids.push(id);
    }

    let mut tree = phylotree::tree::Tree::new();
    // tax_id to NodeID
    let mut id_of: HashMap<i64, usize> = HashMap::new();

    for tax_id in tax_ids {
        let lineage = nwr::get_lineage(&conn, tax_id)?;

        for taxon in lineage.iter() {
            let cur_tax_id = taxon.tax_id;
            if !id_of.contains_key(&cur_tax_id) {
                let node_id = if cur_tax_id == 1 {
                    add_taxon(&mut tree, taxon, None)?
                } else {
                    let parent_tax_id = taxon.parent_tax_id;
                    let parent_id = id_of.get(&parent_tax_id).ok_or_else(|| {
                        anyhow::anyhow!("Parent ID not found: {}", parent_tax_id)
                    })?;
                    add_taxon(&mut tree, taxon, Some(*parent_id))?
                };
                id_of.insert(cur_tax_id, node_id);
            }
        }
    }

    tree.compress()?;
    let out_string = tree.to_newick()?;
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

fn add_taxon(
    tree: &mut phylotree::tree::Tree,
    taxon: &Taxon,
    parent: Option<usize>,
) -> anyhow::Result<usize> {
    let mut node = phylotree::tree::Node::new();
    let name = taxon
        .names
        .get("scientific name")
        .and_then(|v| v.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    node.set_name(name);

    let node_id = if let Some(p) = parent {
        tree.add_child(node, p, None)?
    } else {
        tree.add(node)
    };
    Ok(node_id)
}

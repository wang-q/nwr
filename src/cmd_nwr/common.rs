use clap::*;
use nwr::Taxon;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Output the common tree of terms")
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
                .help("Change working directory"),
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
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };

    let conn = nwr::connect_txdb(&nwrdir).unwrap();

    let mut tax_ids = vec![];
    for term in args.get_many::<String>("terms").unwrap() {
        let id = nwr::term_to_tax_id(&conn, term).unwrap();
        tax_ids.push(id);
    }

    let mut tree = phylotree::tree::Tree::new();
    // tax_id to NodeID
    let mut id_of: HashMap<i64, usize> = HashMap::new();

    for tax_id in tax_ids {
        let lineage = nwr::get_lineage(&conn, tax_id).unwrap();

        for taxon in lineage.iter() {
            let cur_tax_id = taxon.tax_id;
            if !id_of.contains_key(&cur_tax_id) {
                let node_id = if cur_tax_id == 1 {
                    add_taxon(&mut tree, taxon, None)
                } else {
                    let parent_tax_id = taxon.parent_tax_id;
                    let parent_id = id_of.get(&parent_tax_id).unwrap();
                    add_taxon(&mut tree, taxon, Some(*parent_id))
                };
                id_of.insert(cur_tax_id, node_id);
            }
        }
    }

    tree.compress().unwrap();
    let out_string = tree.to_newick().unwrap();
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

fn add_taxon(
    tree: &mut phylotree::tree::Tree,
    taxon: &Taxon,
    parent: Option<usize>,
) -> usize {
    let mut node = phylotree::tree::Node::new();
    let name = taxon.names.get("scientific name").unwrap()[0].clone(); // :S=
    node.set_name(name);
    nwr::add_comment_kv(&mut node, "T", taxon.tax_id.to_string().as_str());
    nwr::add_comment_kv(&mut node, "rank", &taxon.rank);

    if parent.is_some() {
        tree.add_child(node, parent.unwrap(), None).unwrap()
    } else {
        tree.add(node)
    }
}

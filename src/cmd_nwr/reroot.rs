use clap::*;
use phylotree::tree::{Tree};
use std::collections::{BTreeMap, BTreeSet};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("reroot")
        .about("Place the root in the middle of the desired node and its parent")
        .after_help(
            r###"
This tool doesn't provide a complex name matching mechanism, as we expect you
to already have an overview of the tree topology through other tools and
understand your desired outcome

* The node can be either terminal (leaves) or internal
* If multiple nodes are provided, the nodes will be specified as their lowest
  common ancestor
* If the LCA is the original root, print the origianl tree

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .num_args(1)
                .index(1)
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("node")
                .long("node")
                .short('n')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Node name"),
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
    //----------------------------
    // Args
    //----------------------------
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let infile = args.get_one::<String>("infile").unwrap();
    let mut tree = nwr::read_newick(infile);

    // ids with names
    let id_of: BTreeMap<_, _> = nwr::get_name_id(&tree);

    // All IDs matched
    let mut ids = BTreeSet::new();
    for name in args.get_many::<String>("node").unwrap() {
        if id_of.contains_key(name) {
            let id = id_of.get(name).unwrap();
            ids.insert(*id);
        }
    }

    if !ids.is_empty() {
        let mut nodes: Vec<usize> = ids.iter().cloned().collect();
        let mut sub_root_id = nodes.pop().unwrap();

        for id in &nodes {
            sub_root_id = tree.get_common_ancestor(&sub_root_id, id).unwrap();
        }

        let old_root = tree.get_root().unwrap();
        if old_root == sub_root_id {
            let out_string = nwr::format_tree(&tree, "");
            writer.write_all((out_string + "\n").as_ref())?;
            return Ok(());
        }

        let new_root = nwr::insert_parent(&mut tree, &sub_root_id);

        // Swap the nodes from new root to the old root
        // All of them are internal
        let path: Vec<_> = tree
            .get_path_from_root(&new_root)
            .unwrap()
            .into_iter()
            .skip(1)// the first is old root
            .collect();

        let mut edge = None; // root has no edge
        for id in path.iter() {
            edge = nwr::swap_parent(&mut tree, id, edge);
        }

        // can't call compress() now
        let serialized = nwr::format_tree(&tree, "");
        let mut tree = Tree::from_newick(&serialized).unwrap();
        tree.compress().unwrap();
        let out_string = nwr::format_tree(&tree, "");
        writer.write_all((out_string + "\n").as_ref())?;
    }

    Ok(())
}

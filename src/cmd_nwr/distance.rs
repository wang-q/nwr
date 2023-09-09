use clap::*;
use phylotree::tree::{NodeId, Tree};
use std::collections::BTreeMap;
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("distance")
        .about("Output a TSV/phylip file with distances between all named nodes")
        .after_help(
            r###"
Modes and output formats for calculating distances

* root - from each node to the root
    * Node distance
* parent - from each node to their parent
    * Node distance
* pairwise - from each node to each node
    * Node1 Node2   distance
* lca - from nodes in a pair to their lowest common ancestor
    * Node1 Node2   distance1   distance2
    * The definition of LCA here is quite different from "nw_distance"
* phylip - a phylip matrix
    * `-I` and `-L` are both ignored

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
            Arg::new("mode")
                .long("mode")
                .short('m')
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("root"),
                    builder::PossibleValue::new("parent"),
                    builder::PossibleValue::new("pairwise"),
                    builder::PossibleValue::new("lca"),
                    builder::PossibleValue::new("phylip"),
                ])
                .default_value("root")
                .help("Set the mode for calculating distances"),
        )
        .arg(
            Arg::new("Internal")
                .long("Internal")
                .short('I')
                .action(ArgAction::SetTrue)
                .help("Ignore internal nodes"),
        )
        .arg(
            Arg::new("Leaf")
                .long("Leaf")
                .short('L')
                .action(ArgAction::SetTrue)
                .help("Ignore leaf nodes"),
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

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = nwr::read_newick(infile);

    let mode = args.get_one::<String>("mode").unwrap();

    let skip_internal = args.get_flag("Internal");
    let skip_leaf = args.get_flag("Leaf");

    // ids with names
    let mut id_of = BTreeMap::new();
    tree.inorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .for_each(|id| {
            let node = tree.get(id).unwrap();
            if let Some(x) = &node.name {
                if node.is_tip() && !skip_leaf {
                    id_of.insert(x.clone(), *id);
                }
                if !node.is_tip() && !skip_internal {
                    id_of.insert(x.clone(), *id);
                }
            }
        });

    match mode.as_str() {
        "root" => dist_root(&tree, &id_of, &mut writer),
        "parent" => dist_parent(tree, &id_of, &mut writer),
        "pairwise" => dist_pairwise(tree, &id_of, &mut writer),
        "lca" => dist_lca(tree, &id_of, &mut writer),
        "phylip" => {
            let matrix = tree.distance_matrix().unwrap();
            writer
                .write_fmt(format_args!("{}", matrix.to_phylip(true).unwrap()))
                .unwrap();
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn dist_root(tree: &Tree, id_of: &BTreeMap<String, NodeId>, writer: &mut Box<dyn Write>) {
    let root = tree.get_root().unwrap();
    for (k, v) in id_of.iter() {
        let dist = {
            let (edge_sum, num_edges) = tree.get_distance(&root, v).unwrap();
            match edge_sum {
                Some(height) => height,
                None => num_edges as f64,
            }
        };
        writer.write_fmt(format_args!("{}\t{}\n", k, dist)).unwrap();
    }
}

fn dist_parent(tree: Tree, id_of: &BTreeMap<String, NodeId>, writer: &mut Box<dyn Write>) {
    for (k, v) in id_of.iter() {
        let parent = tree.get(v).unwrap().parent;
        if parent.is_none() {
            continue;
        }
        let parent = parent.unwrap();

        let dist = {
            let (edge_sum, num_edges) = tree.get_distance(&parent, v).unwrap();
            match edge_sum {
                Some(height) => height,
                None => num_edges as f64,
            }
        };
        writer.write_fmt(format_args!("{}\t{}\n", k, dist)).unwrap();
    }
}

fn dist_pairwise(tree: Tree, id_of: &BTreeMap<String, NodeId>, writer: &mut Box<dyn Write>) {
    for (k1, v1) in id_of.iter() {
        for (k2, v2) in id_of.iter() {
            let dist = {
                let (edge_sum, num_edges) = tree.get_distance(v1, v2).unwrap();
                match edge_sum {
                    Some(height) => height,
                    None => num_edges as f64,
                }
            };
            writer
                .write_fmt(format_args!("{}\t{}\t{}\n", k1, k2, dist))
                .unwrap();
        }
    }
}

fn dist_lca(tree: Tree, id_of: &BTreeMap<String, NodeId>, writer: &mut Box<dyn Write>) {
    for (k1, v1) in id_of.iter() {
        for (k2, v2) in id_of.iter() {
            let lca = tree.get_common_ancestor(v1, v2).unwrap();

            let dist1 = {
                let (edge_sum, num_edges) = tree.get_distance(&lca, v1).unwrap();
                match edge_sum {
                    Some(height) => height,
                    None => num_edges as f64,
                }
            };

            let dist2 = {
                let (edge_sum, num_edges) = tree.get_distance(&lca, v2).unwrap();
                match edge_sum {
                    Some(height) => height,
                    None => num_edges as f64,
                }
            };
            writer
                .write_fmt(format_args!("{}\t{}\t{}\t{}\n", k1, k2, dist1, dist2))
                .unwrap();
        }
    }
}

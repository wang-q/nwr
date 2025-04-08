use clap::*;
use phylotree::tree::{EdgeLength, Node, Tree};
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("upgma")
        .about("Build a tree using UPGMA algorithm")
        .after_help(
            r###"
Build a rooted phylogenetic tree using the UPGMA (Unweighted Pair Group Method with Arithmetic Mean) algorithm.

Input format:
    PHYLIP distance matrix format

Examples:
    nwr build upgma input.phy

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input file phylip distance matrix"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
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
    let infile = args.get_one::<String>("infile").unwrap();
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    //----------------------------
    // Ops
    //----------------------------
    // Load matrix from pairwise distances
    let matrix = intspan::NamedMatrix::from_relaxed_phylip(infile);
    let names = matrix.get_names();
    let size = matrix.size();

    // Create tree with root
    let mut tree = Tree::new();
    let root_id = tree.add(Node::new());

    // Initialize leaf nodes
    let mut nodes: Vec<Option<(usize, usize)>> = names
        .iter()
        .map(|name| {
            let node = Node::new_named(name);
            let node_id = tree.add_child(node, root_id, None).unwrap();
            Some((node_id, 1))
        })
        .collect();

    // Create symmetric distance matrix
    let mut dist_matrix = vec![vec![0.0; size]; size];
    for i in 0..size {
        for j in 0..i {
            let dist = matrix.get(i, j) as EdgeLength;
            dist_matrix[i][j] = dist;
            dist_matrix[j][i] = dist;
        }
    }

    // Main UPGMA loop
    while nodes.iter().filter_map(|x| x.as_ref()).count() > 1 {
        // Find minimum distance pair
        let (min_dist, min_i, min_j) = {
            let mut min_dist = f64::MAX;
            let mut min_i = 0;
            let mut min_j = 0;

            // Only need to check lower triangular matrix
            for i in 0..size {
                for j in 0..i {
                    if nodes[i].is_some() && nodes[j].is_some() {
                        let dist = dist_matrix[i][j];
                        if dist < min_dist {
                            min_dist = dist;
                            min_i = i;
                            min_j = j;
                        }
                    }
                }
            }
            (min_dist, min_i, min_j)
        };

        // Merge nodes
        if let (Some((node_i, size_i)), Some((node_j, size_j))) =
            (nodes[min_i].take(), nodes[min_j].take())
        {
            let height = min_dist / 2.0;

            // Calculate branch lengths
            let edge_i = height - nwr::node_height(&tree, &node_i);
            let edge_j = height - nwr::node_height(&tree, &node_j);

            // Update tree
            tree.get_mut(&node_i)?.parent_edge = Some(edge_i);
            tree.get_mut(&node_j)?.parent_edge = Some(edge_j);

            let new_node = nwr::insert_parent_pair(&mut tree, &node_i, &node_j);

            // Update distances
            let new_size = size_i + size_j;
            for k in 0..size {
                if k != min_i && k != min_j && nodes[k].is_some() {
                    let dist_ik = dist_matrix[k.max(min_i)][k.min(min_i)];
                    let dist_jk = dist_matrix[k.max(min_j)][k.min(min_j)];

                    let new_dist = (size_i as f64 * dist_ik + size_j as f64 * dist_jk)
                        / new_size as f64;

                    if k > min_i {
                        dist_matrix[k][min_i] = new_dist;
                    } else {
                        dist_matrix[min_i][k] = new_dist;
                    }
                }
            }

            // Clear distances of min_j
            for k in 0..size {
                dist_matrix[k][min_j] = f64::MAX;
                dist_matrix[min_j][k] = f64::MAX;
            }

            nodes[min_j] = None;
            nodes[min_i] = Some((new_node, new_size));
        }
    }

    // Remove redundant root
    let root = tree.get_root()?;
    let root_children = tree.get(&root)?.children.clone();
    if root_children.len() == 1 {
        nwr::delete_node(&mut tree, &root_children[0])?;
    }

    // eprintln!("tree = {:#?}", tree);

    // Output Newick format
    writer.write_fmt(format_args!("{}\n", tree.to_newick()?))?;
    Ok(())
}

use clap::*;
use phylotree::tree::{EdgeLength, Node, Tree};
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("nj")
        .about("Build a phylogenetic tree using Neighbor-Joining algorithm")
        .after_help(
            r###"
Build a phylogenetic tree using the Neighbor-Joining (NJ) algorithm.

* NJ is a bottom-up clustering method that does not assume a constant rate of evolution
* NJ produces an unrooted tree, but for convenience, this implementation places a root at the last joined node
* The algorithm minimizes the total branch length at each step of the clustering

Input format:
    PHYLIP distance matrix format

Examples:
    nwr build nj input.phy

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
    // Load distance matrix
    let matrix = intspan::NamedMatrix::from_relaxed_phylip(infile);
    let names = matrix.get_names();
    let size = matrix.size();

    // Create tree with root
    let mut tree = Tree::new();
    let root_id = tree.add(Node::new());

    // Initialize leaf nodes
    let mut nodes: Vec<Option<usize>> = names
        .iter()
        .map(|name| {
            let node = Node::new_named(name);
            let node_id = tree.add_child(node, root_id, None).unwrap();
            Some(node_id)
        })
        .collect();

    // Create distance matrix
    let mut dist_matrix = vec![vec![0.0; size]; size];
    for i in 0..size {
        for j in 0..size {
            dist_matrix[i][j] = matrix.get(i, j) as EdgeLength;
        }
    }

    // Main NJ loop
    while nodes.iter().filter_map(|x| *x).count() > 2 {
        // Calculate r values
        let mut r = vec![0.0; size];
        for i in 0..size {
            if nodes[i].is_none() {
                continue;
            }
            r[i] = nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| n.is_some())
                .map(|(j, _)| dist_matrix[i][j])
                .sum::<EdgeLength>();
        }
        let active_nodes = nodes.iter().filter_map(|x| *x).count();

        // Find minimum Q-value pair
        let mut min_q = EdgeLength::MAX;
        let mut min_i = 0;
        let mut min_j = 0;
        for i in 0..size {
            if nodes[i].is_none() {
                continue;
            }
            for j in 0..i {
                if nodes[j].is_none() {
                    continue;
                }
                let q =
                    (active_nodes as EdgeLength - 2.0) * dist_matrix[i][j] - r[i] - r[j];
                if q < min_q {
                    min_q = q;
                    min_i = i;
                    min_j = j;
                }
            }
        }

        // Merge nodes
        if let (Some(node_i), Some(node_j)) = (nodes[min_i], nodes[min_j]) {
            // Calculate branch lengths
            let dist_ij = dist_matrix[min_i][min_j];
            let edge_i = dist_ij / 2.0
                + (r[min_i] - r[min_j]) / (2.0 * (active_nodes as EdgeLength - 2.0));
            let edge_j = dist_ij - edge_i;

            tree.get_mut(&node_i).unwrap().parent_edge = Some(edge_i);
            tree.get_mut(&node_j).unwrap().parent_edge = Some(edge_j);

            let new_node = nwr::insert_parent_pair(&mut tree, &node_i, &node_j);
            nodes[min_j] = None;
            nodes[min_i] = Some(new_node);

            // Update distances to the new node (store in position min_i)
            for k in 0..size {
                if k != min_i && k != min_j && nodes[k].is_some() {
                    let new_dist = (dist_matrix[min_i][k] + dist_matrix[min_j][k]
                        - dist_matrix[min_i][min_j])
                        / 2.0;
                    dist_matrix[k][min_i] = new_dist;
                    dist_matrix[min_i][k] = new_dist;
                }
            }

            // Update nodes vector
            nodes[min_j] = None;
            nodes[min_i] = Some(new_node);
        }
    }

    // Join the last two nodes
    let remaining: Vec<(usize, usize)> = nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, &node)| node.map(|n| (idx, n)))
        .collect();
    // eprintln!("remaining = {:#?}", remaining);

    if remaining.len() == 2 {
        let dist = dist_matrix[remaining[0].0][remaining[1].0];
        let edge_len = dist / 2.0;

        tree.get_mut(&remaining[0].1).unwrap().parent_edge = Some(edge_len);
        tree.get_mut(&remaining[1].1).unwrap().parent_edge = Some(edge_len);
    }

    // eprintln!("tree = {:#?}", tree);

    // Output Newick format
    writer.write_fmt(format_args!("{}\n", tree.to_newick()?))?;
    Ok(())
}

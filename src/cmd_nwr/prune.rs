use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("prune")
        .about("Remove nodes from the Newick file")
        .after_help(
            r###"
This tool removes nodes whose labels matching the following rules

* The intersection between the nodes in the tree and the provided
* Nodes matching the case insensitive regular expression(s)

For more complex needs, you can use `nwr label` to generate a list,
then pass it in with `--file`.

Removing nodes results in a change in the topology of the tree,
and for internal nodes, the following additional operations are performed

* If removing a node causes its parent to have only one child,
  the parent is spliced out and the remaining child is attached to its grandparent
* If an internal node loses all its children, that node will also be removed

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
            Arg::new("file")
                .long("file")
                .num_args(1)
                .help("A file contains node names"),
        )
        .arg(
            Arg::new("regex")
                .long("regex")
                .short('r')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Nodes match the regular expression"),
        )
        .arg(
            Arg::new("descendants")
                .long("descendants")
                .short('D')
                .action(ArgAction::SetTrue)
                .help("Include all descendants of internal nodes")
                .hide(true),
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

    //----------------------------
    // Operating
    //----------------------------
    // record current internals
    let internals: Vec<_> = get_internals(&tree);

    // All IDs matching names
    let ids = nwr::match_names(&tree, args);
    for id in ids.iter() {
        tree.prune(id).unwrap();
    }

    // remove new leaves, which were internal nodes
    let new_internals: Vec<_> = get_internals(&tree);

    for id in internals.iter() {
        if *id != 0 && !new_internals.contains(id) {
            tree.prune(id).unwrap();
        }
    }

    tree.compress().unwrap();

    //----------------------------
    // Output
    //----------------------------
    let out_string = tree.to_newick().unwrap();
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

fn get_internals(tree: &phylotree::tree::Tree) -> Vec<usize> {
    let mut ids = vec![];
    tree.preorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .for_each(|id| {
            let node = tree.get(id).unwrap();
            if !node.is_tip() {
                ids.push(*id);
            }
        });

    ids
}

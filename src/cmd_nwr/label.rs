use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("label")
        .about("Labels in the Newick file")
        .after_help(
            r###"
This tool selectively outputs the names of the nodes in the tree

* The intersection between the nodes in the tree and the provided
* Nodes matching the regular expression(s)

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
            Arg::new("Internal")
                .long("Internal")
                .short('I')
                .action(ArgAction::SetTrue)
                .help("Don't print internal labels"),
        )
        .arg(
            Arg::new("Leaf")
                .long("Leaf")
                .short('L')
                .action(ArgAction::SetTrue)
                .help("Don't print leaf labels"),
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

    // All IDs matching names
    let ids_pos = nwr::match_positions(&tree, args);

    // All IDs matching names
    let ids_name = nwr::match_names(&tree, args);

    // Default is printing all named nodes
    let ids: Vec<usize> = if ids_name.is_empty() {
        ids_pos.into_iter().collect()
    } else {
        ids_pos.intersection(&ids_name).cloned().collect()
    };

    for id in ids.iter() {
        let node = tree.get(id).unwrap();
        if let Some(x) = node.name.clone() {
            writer.write_fmt(format_args!("{}\n", x)).unwrap();
        }
    }

    Ok(())
}

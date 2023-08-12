use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("comment")
        .about("Add comments to node(s) in a Newick file")
        .after_help(
            r###"
* Comments are in the NHX-like format
    * :key=value

* For nodes with names, set `--node` to the name
* For nodes without names (e.g., internal nodes), set `--lca` to a combination
  of the node names, separated by commas
    * `--lca A,B`

* Set `--string` to add free-form strings

* The following options are used for visualization
    * `--label`, `--color` and `--comment` take 1 argument
    * `--dot` and `--bar` take 1 or 0 argument

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
                .help("Node name, or lowest common ancestor of nodes"),
        )
        .arg(
            Arg::new("lca")
                .long("lca")
                .short('l')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Lowest common ancestor of nodes"),
        )
        .arg(
            Arg::new("string")
                .long("string")
                .short('s')
                .num_args(1)
                .help("Free-form strings"),
        )
        .arg(
            Arg::new("label")
                .long("label")
                .num_args(1)
                .help("Use this text instead of the default two spaces"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .num_args(1)
                .help("Use this text instead of the default two spaces"),
        )
        .arg(
            Arg::new("comment")
                .long("comment")
                .num_args(1)
                .help("Use this text instead of the default two spaces"),
        )
        .arg(
            Arg::new("dot")
                .long("dot")
                .num_args(0..=1)
                .help("Use this text instead of the default two spaces"),
        )
        .arg(
            Arg::new("bar")
                .long("bar")
                .num_args(0..=1)
                .help("Use this text instead of the default two spaces"),
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

    let label = args.get_one::<String>("label");
    let color = args.get_one::<String>("color");

    let infile = args.get_one::<String>("infile").unwrap();
    let mut tree = nwr::read_newick(infile);

    // let names = tree.

    let mut nodes = vec![];
    if args.contains_id("node") {
        for node in args.get_many::<String>("node").unwrap() {
            nodes.push(node.to_string());
        }
    }

    let out_string = nwr::format_tree(&tree, "");
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

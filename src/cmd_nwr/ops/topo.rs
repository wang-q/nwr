use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("topo")
        .about("Topological information of the Newick file")
        .after_help(
            r###"
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
            Arg::new("bl")
                .long("bl")
                .action(ArgAction::SetTrue)
                .help("Keep branch lengths"),
        )
        .arg(
            Arg::new("comment")
                .long("comment")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Keep comments"),
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

    let is_bl = args.get_flag("bl");
    let is_comment = args.get_flag("comment");
    let skip_internal = args.get_flag("Internal");
    let skip_leaf = args.get_flag("Leaf");

    let infile = args.get_one::<String>("infile").unwrap();
    let mut tree = nwr::read_newick(infile);

    // inorder trigger IsNotBinary
    tree.preorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .for_each(|id| {
            let node = tree.get_mut(id).unwrap();

            if !is_bl {
                node.parent_edge = None;
            }
            if !is_comment {
                node.comment = None;
            }
            if node.is_tip() && skip_leaf {
                node.name = None;
            }
            if !node.is_tip() && skip_internal {
                node.name = None;
            }
        });

    let out_string = nwr::format_tree(&tree, "");
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("label")
        .about("Labels in the Newick file")
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

    let skip_internal = args.get_flag("Internal");
    let skip_leaf = args.get_flag("Leaf");

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = nwr::read_newick(infile);

    tree.inorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .for_each(|id| {
            let node = tree.get(id).unwrap();
            if let Some(x) = node.name.clone() {
                if node.is_tip() {
                    if !skip_leaf {
                        writer.write_all((x + "\n").as_ref()).unwrap();
                    }
                } else {
                    if !skip_internal {
                        writer.write_all((x + "\n").as_ref()).unwrap();
                    }
                }
            }
        });

    Ok(())
}

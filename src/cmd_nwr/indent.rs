use clap::*;
use log::warn;
use phylotree::tree::{Node, NodeId, Tree};
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("indent")
        .about("Indent the Newick file")
        .after_help(
            r###"
* Set `--text` to something other than whitespaces will result in an invalid Newick file

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
            Arg::new("text")
                .long("text")
                .short('t')
                .num_args(1)
                .default_value("    ")
                .help("Use this text instead of the default four spaces"),
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

    let text = args.get_one::<String>("text").unwrap();

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = read_newick(infile);

    eprintln!("tree = {:#?}", tree);
    eprintln!("tree = {:#?}", tree.to_newick().unwrap());
    eprintln!("tree = {:#?}", format_tree(&tree));

    // let ids: Vec<_> = tree.levelorder(&tree.get_root().unwrap())
    //     .unwrap()
    //     .iter()
    //     .map(|id| *id)
    //     .collect();
    //
    // eprintln!("ids = {:#?}", ids);

    Ok(())
}

fn read_newick(infile: &str) -> Tree {
    let mut reader = intspan::reader(infile);
    let mut newick = String::new();
    reader.read_to_string(&mut newick).expect("Read error");
    let mut tree = Tree::from_newick(newick.as_str()).unwrap();

    // Remove leading and trailing whitespaces of node names
    tree.preorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .for_each(|id| {
            let node = tree.get_mut(id).unwrap();
            if node.name.is_some() {
                node.set_name(node.name.clone().unwrap().trim().to_string());
            }
        });

    tree
}

fn format_tree(tree: &Tree) -> String {
    let root = tree.get_root().unwrap();
    format_subtree(tree, &root) + ";"
}

fn format_subtree(tree: &Tree, root: &NodeId) -> String {
    let root = tree.get(root).unwrap();
    let children = {
        let children = &root.children;
        if children.is_empty() {
            format_node(root)
        } else {
            let branch_set = children
                .into_iter()
                .map(|child| format_subtree(tree, child))
                .collect::<Vec<_>>()
                .join(",");
            format!("({}){}", branch_set, format_node(root))
        }
    };

    children
}

fn format_node(node: &Node) -> String {
    let mut repr = String::new();
    if let Some(name) = node.name.clone() {
        repr += &name;
    }
    if let Some(comment) = node.comment.clone() {
        repr += &format!("[{}]", &comment);
    }
    if let Some(parent_edge) = node.parent_edge {
        repr += &format!(":{}", &parent_edge);
    }

    repr
}

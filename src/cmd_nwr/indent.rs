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
    * Use `--text ".   "` can produce visual guide lines

* Set `--text` to empty ("") will remove indentation

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
                .default_value("  ")
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

    let text = args.get_one::<String>("text").unwrap();

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = read_newick(infile);

    // eprintln!("tree = {:#?}", tree);

    // let ids: Vec<_> = tree.levelorder(&tree.get_root().unwrap())
    //     .unwrap()
    //     .iter()
    //     .map(|id| *id)
    //     .collect();
    //
    // eprintln!("ids = {:#?}", ids);

    let out_string = format_tree(&tree, text);
    writer.write_all((out_string + "\n").as_ref())?;

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

fn format_tree(tree: &Tree, indent: &str) -> String {
    let root = tree.get_root().unwrap();
    format_subtree(tree, &root, indent) + ";"
}

fn format_subtree(tree: &Tree, cur_id: &NodeId, indent: &str) -> String {
    let cur_node = tree.get(cur_id).unwrap();
    let formatted = {
        let children = &cur_node.children;
        let depth = cur_node.get_depth();
        if children.is_empty() {
            if indent.is_empty() {
                format_node(cur_node)
            } else {
                let indention = indent.repeat(depth);
                format!("{}{}", indention, format_node(cur_node))
            }
        } else {
            let branch_set = children
                .into_iter()
                .map(|child| format_subtree(tree, child, indent))
                .collect::<Vec<_>>();

            if indent.is_empty() {
                format!("({}){}", branch_set.join(","), format_node(cur_node))
            } else {
                let root = tree.get_root().unwrap();
                let indention = indent.repeat(depth);
                format!("{}(\n{}\n{}){}", indention, branch_set.join(",\n"), indention, format_node(cur_node))
            }
        }
    };

    formatted
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

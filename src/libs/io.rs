use phylotree::tree::{Node, NodeId, Tree};
use std::io::Read;
use std::str;

//----------------------------
// newick
//----------------------------
pub fn read_newick(infile: &str) -> Tree {
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
                let name = node.name.clone().unwrap().trim().to_string();
                if name.is_empty() {
                    node.name = None;
                } else {
                    node.set_name(node.name.clone().unwrap().trim().to_string());
                }
            }
        });

    tree
}

/// Writes the tree with indentations
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "(A,B);";
/// let tree = Tree::from_newick(newick).unwrap();
///
/// assert_eq!(nwr::format_tree(&tree, "  "), "(\n  A,\n  B\n);".to_string());
/// ```
pub fn format_tree(tree: &Tree, indent: &str) -> String {
    let root = tree.get_root().unwrap();
    format_subtree(tree, &root, indent) + ";"
}

pub fn format_subtree(tree: &Tree, id: &NodeId, indent: &str) -> String {
    let node = tree.get(id).unwrap();

    let children = &node.children;
    let depth = node.get_depth();

    if children.is_empty() {
        if indent.is_empty() {
            format_node(node)
        } else {
            let indention = indent.repeat(depth);
            format!("{}{}", indention, format_node(node))
        }
    } else {
        let branch_set = children
            .iter()
            .map(|child| format_subtree(tree, child, indent))
            .collect::<Vec<_>>();

        if indent.is_empty() {
            format!("({}){}", branch_set.join(","), format_node(node))
        } else {
            let indention = indent.repeat(depth);
            format!(
                "{}(\n{}\n{}){}",
                indention,
                branch_set.join(",\n"),
                indention,
                format_node(node)
            )
        }
    }
}

fn format_node(node: &Node) -> String {
    let mut repr = String::new();
    if let Some(name) = node.name.clone() {
        repr += &name;
    }
    if let Some(parent_edge) = node.parent_edge {
        repr += &format!(":{}", &parent_edge);
    }
    if let Some(comment) = node.comment.clone() {
        repr += &format!("[{}]", &comment);
    }

    repr
}

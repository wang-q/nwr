use phylotree::tree::{Node, NodeId, Tree};
use regex::RegexBuilder;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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

/// Sort the children of each node by alphanumeric order of labels
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "(A,B);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// nwr::order_tree_an(&mut tree, "anr");
/// assert_eq!(tree.to_newick().unwrap(), "(B,A);".to_string());
/// ```
pub fn order_tree_an(tree: &mut Tree, opt: &str) {
    let root = tree.get_root().unwrap();

    let ids = tree.levelorder(&root).unwrap().to_vec();

    let mut an_of: HashMap<NodeId, String> = HashMap::new();
    for id in &ids {
        let node = tree.get(id).unwrap();
        let name = &node.name;
        if name.is_none() {
            an_of.insert(*id, "".to_string());
        } else {
            an_of.insert(*id, name.clone().unwrap());
        }
    }

    for id in &ids {
        let node = tree.get_mut(id).unwrap();
        let children = &mut node.children;
        if children.is_empty() {
            continue;
        } else {
            match opt {
                "an" => {
                    children.sort_by(|a, b| an_of.get(a).unwrap().cmp(an_of.get(b).unwrap()));
                }
                "anr" => {
                    children.sort_by(|a, b| an_of.get(b).unwrap().cmp(an_of.get(a).unwrap()));
                }
                _ => panic!("Invalid opt"),
            }
        }
    }
}

/// Sort the children of each node by number of descendants
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "((A,B),C);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// nwr::order_tree_nd(&mut tree, "nd");
/// assert_eq!(tree.to_newick().unwrap(), "(C,(A,B));".to_string());
/// ```
pub fn order_tree_nd(tree: &mut Tree, opt: &str) {
    let root = tree.get_root().unwrap();

    let ids = tree.levelorder(&root).unwrap().to_vec();

    let mut nd_of: HashMap<NodeId, usize> = HashMap::new();
    for id in &ids {
        let node = tree.get(id).unwrap();
        let children = &node.children;
        if children.is_empty() {
            nd_of.insert(*id, 0);
        } else {
            let nd = tree.get_descendants(id).unwrap();
            nd_of.insert(*id, nd.len());
        }
    }

    for id in &ids {
        let node = tree.get_mut(id).unwrap();
        let children = &mut node.children;
        if children.is_empty() {
            continue;
        } else {
            match opt {
                "nd" => {
                    children.sort_by(|a, b| nd_of.get(a).unwrap().cmp(nd_of.get(b).unwrap()));
                }
                "ndr" => {
                    children.sort_by(|a, b| nd_of.get(b).unwrap().cmp(nd_of.get(a).unwrap()));
                }
                _ => panic!("Invalid opt"),
            }
        }
    }
}

/// Get node names
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "((A,B)D,C);";
/// let tree = Tree::from_newick(newick).unwrap();
/// nwr::get_names(&tree);
/// assert_eq!(nwr::get_names(&tree), vec!["D".to_string(),"A".to_string(),"B".to_string(),"C".to_string(), ]);
/// ```
pub fn get_names(tree: &Tree) -> Vec<String> {
    let names: Vec<_> = tree
        .preorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .map(|id| tree.get(id).unwrap())
        .filter_map(|node| node.name.clone().map(|_| node.name.clone().unwrap()))
        .collect::<Vec<String>>();

    names
}

/// Get hash of name-id
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "((A,B),C);";
/// let tree = Tree::from_newick(newick).unwrap();
/// let id_of = nwr::get_name_id(&tree);
/// assert_eq!(*id_of.get("A").unwrap(), 2usize);
/// ```
pub fn get_name_id(tree: &Tree) -> BTreeMap<String, usize> {
    let mut id_of = BTreeMap::new();
    for id in tree.preorder(&tree.get_root().unwrap()).unwrap().iter() {
        let node = tree.get(id).unwrap();
        let name = node.name.clone();
        if let Some(x) = name {
            id_of.insert(x, *id);
        }
    }

    id_of
}

/// Adds key-value comments to a node
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "(A,B);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let mut node = tree.get_by_name_mut("A").unwrap();
/// nwr::add_comment_kv(&mut node, "color", "red");
///
/// assert_eq!(tree.to_newick().unwrap(), "(A[color=red],B);".to_string());
/// ```
pub fn add_comment_kv(node: &mut Node, key: &str, value: &str) {
    let comment = match &node.comment {
        None => format!("{}={}", key, value),
        Some(x) => format!("{}:{}={}", x, key, value),
    };

    node.comment = Some(comment);
}

/// Adds key-value comments to a node
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "(A,B);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let mut node = tree.get_by_name_mut("A").unwrap();
/// nwr::add_comment(&mut node, "color=red");
///
/// assert_eq!(tree.to_newick().unwrap(), "(A[color=red],B);".to_string());
/// ```
pub fn add_comment(node: &mut Node, c: &str) {
    let comment = match &node.comment {
        None => c.to_string(),
        Some(x) => format!("{}:{}", x, c),
    };

    node.comment = Some(comment);
}

pub fn match_names(id_of: &BTreeMap<String, usize>, args: &clap::ArgMatches) -> BTreeSet<usize> {
    // all matched IDs
    let mut ids = BTreeSet::new();

    // ids supplied by --node
    if args.contains_id("node") {
        for name in args.get_many::<String>("node").unwrap() {
            if id_of.contains_key(name) {
                let id = id_of.get(name).unwrap();
                ids.insert(*id);
            }
        }
    }

    // ids supplied by --file
    if args.contains_id("file") {
        let file = args.get_one::<String>("file").unwrap();
        for name in intspan::read_first_column(file).iter() {
            if id_of.contains_key(name) {
                let id = id_of.get(name).unwrap();
                ids.insert(*id);
            }
        }
    }

    // ids matched with --regex
    if args.contains_id("regex") {
        for regex in args.get_many::<String>("regex").unwrap() {
            let re = RegexBuilder::new(regex)
                .case_insensitive(true)
                .unicode(false)
                .build()
                .unwrap();
            for name in id_of.keys() {
                if re.is_match(name) {
                    let id = id_of.get(name).unwrap();
                    ids.insert(*id);
                }
            }
        }
    }

    ids
}

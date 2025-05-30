use intspan::IntSpan;
use phylotree::tree::{EdgeLength, Node, NodeId, Tree};
use regex::RegexBuilder;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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
                    children.sort_by(|a, b| {
                        an_of.get(a).unwrap().cmp(an_of.get(b).unwrap())
                    });
                }
                "anr" => {
                    children.sort_by(|a, b| {
                        an_of.get(b).unwrap().cmp(an_of.get(a).unwrap())
                    });
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
                    children.sort_by(|a, b| {
                        nd_of.get(a).unwrap().cmp(nd_of.get(b).unwrap())
                    });
                }
                "ndr" => {
                    children.sort_by(|a, b| {
                        nd_of.get(b).unwrap().cmp(nd_of.get(a).unwrap())
                    });
                }
                _ => panic!("Invalid opt"),
            }
        }
    }
}

/// Sort the children of each node by a list of names
///
/// ```
/// use phylotree::tree::Tree;
///
/// // Simple case with only leaf nodes
/// let newick = "(A,B,C);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// nwr::order_tree_list(&mut tree, &["C".to_string(), "B".to_string(), "A".to_string()]);
/// assert_eq!(tree.to_newick().unwrap(), "(C,B,A);".to_string());
///
/// // Case with internal nodes
/// let newick = "((A,B),(C,D));";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// nwr::order_tree_list(&mut tree, &["C".to_string(), "B".to_string(), "A".to_string()]);
/// assert_eq!(tree.to_newick().unwrap(), "((C,D),(B,A));".to_string());
///
/// // Case with internal nodes and names
/// let newick = "((A,B)X,(C,D)Y);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// nwr::order_tree_list(&mut tree, &["C".to_string(), "B".to_string(), "A".to_string()]);
/// assert_eq!(tree.to_newick().unwrap(), "((C,D)Y,(B,A)X);".to_string());
///
/// // Case with unlisted nodes
/// let newick = "((A,B),(C,E));";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// nwr::order_tree_list(&mut tree, &["C".to_string(), "B".to_string()]);
/// assert_eq!(tree.to_newick().unwrap(), "((C,E),(B,A));".to_string());
/// ```
pub fn order_tree_list(tree: &mut Tree, opt: &[String]) {
    let root = tree.get_root().unwrap();
    let ids = tree.levelorder(&root).unwrap().to_vec();

    // Create a mapping from name to position
    let mut pos_of: HashMap<String, usize> = HashMap::new();
    for (idx, name) in opt.iter().enumerate() {
        pos_of.insert(name.clone(), idx);
    }

    // Create a mapping from node ID to sort position
    let mut order_of: HashMap<NodeId, usize> = HashMap::new();

    // First pass: assign positions to named nodes that are in the list
    for id in &ids {
        let node = tree.get(id).unwrap();
        if let Some(name) = &node.name {
            if let Some(&pos) = pos_of.get(name) {
                order_of.insert(*id, pos);
            }
        }
    }

    // Second pass: assign positions to all other nodes based on their children
    for id in &ids {
        if !order_of.contains_key(id) {
            let default_pos = opt.len();
            let descendants = tree.get_descendants(id).unwrap();
            let min_child_pos = descendants
                .iter()
                .map(|child_id| *order_of.get(child_id).unwrap_or(&default_pos))
                .min()
                .unwrap_or(default_pos);
            order_of.insert(*id, min_child_pos);
        }
    }

    // Sort children of each node
    for id in &ids {
        let node = tree.get_mut(id).unwrap();
        let children = &mut node.children;
        if !children.is_empty() {
            children.sort_by(|a, b| {
                let pos_a = order_of.get(a).unwrap_or(&usize::MAX);
                let pos_b = order_of.get(b).unwrap_or(&usize::MAX);

                if pos_a == pos_b {
                    a.cmp(b)
                } else {
                    pos_a.cmp(pos_b)
                }
            });
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

/// Retrieves value from comments of a node
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "(A[T=9606:S=Homo sapiens],B);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let mut node = tree.get_by_name_mut("A").unwrap();
/// let sciname = nwr::get_comment_k(&node, "S");
///
/// assert_eq!(sciname.unwrap(), "Homo sapiens".to_string());
/// ```
pub fn get_comment_k(node: &Node, key: &str) -> Option<String> {
    let mut value: Option<String> = None;
    if let Some(comment) = node.comment.clone() {
        let parts: Vec<&str> = comment.split(':').collect();

        for pt in parts {
            let key = format!("{}=", key);
            if pt.starts_with(&key) {
                value = Some(pt.replace(&key, "").to_string());
            }
        }
    }

    value
}

/// Get hash of id-comment
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "((A[S=Human],B),C);";
/// let tree = Tree::from_newick(newick).unwrap();
/// let comment_of = nwr::get_id_comment(&tree);
/// assert_eq!(comment_of.get(&2usize).unwrap(), "S=Human");
/// ```
pub fn get_id_comment(tree: &Tree) -> BTreeMap<usize, String> {
    let mut comment_of = BTreeMap::new();
    for id in tree.preorder(&tree.get_root().unwrap()).unwrap().iter() {
        let node = tree.get(id).unwrap();
        let comment = node.comment.clone();
        if let Some(x) = comment {
            comment_of.insert(*id, x);
        }
    }

    comment_of
}

/// Insert a node in the middle of the desired node and its parent
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "((A,B),C);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id = tree.get_by_name("B").unwrap().id;
///
/// nwr::insert_parent(&mut tree, &id);
///
/// assert_eq!(tree.to_newick().unwrap(), "((A,(B)),C);".to_string());
///
/// let newick = "((A:1,B:1):1,C:1);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id = tree.get_by_name("B").unwrap().id;
///
/// nwr::insert_parent(&mut tree, &id);
///
/// assert_eq!(tree.to_newick().unwrap(), "((A:1,(B:0.5):0.5):1,C:1);".to_string());
/// ```
pub fn insert_parent(tree: &mut Tree, id: &NodeId) -> NodeId {
    let parent = tree.get(id).unwrap().parent.unwrap();
    let new_edge = tree.get(id).unwrap().parent_edge.map(|p| p / 2.0);

    let new = tree.add_child(Node::new(), parent, new_edge).unwrap();

    tree.get_mut(id).unwrap().set_parent(new, new_edge);
    tree.get_mut(&new).unwrap().add_child(*id, new_edge);

    tree.get_mut(&parent).unwrap().remove_child(id).unwrap();

    new
}

/// Swap parent-child link of a node
///
/// ```
/// use phylotree::tree::Tree;
///
/// let newick = "((A,B)D,C)E;";
/// let mut tree = Tree::from_newick(newick).unwrap();
///
/// let id_b = tree.get_by_name("B").unwrap().id;
/// let id_d = tree.get_by_name("D").unwrap().id;
/// let new_root = nwr::insert_parent(&mut tree, &id_b);
///
/// let mut edge = None;
/// edge = nwr::swap_parent(&mut tree, &id_d, edge);
/// edge = nwr::swap_parent(&mut tree, &new_root, edge);
///
/// assert_eq!(tree.to_newick().unwrap(), "(B,(A,(C)E)D);".to_string());
/// ```
pub fn swap_parent(
    tree: &mut Tree,
    id: &NodeId,
    prev_edge: Option<EdgeLength>,
) -> Option<EdgeLength> {
    let parent = tree.get(id).unwrap().parent.unwrap();

    tree.get_mut(id).unwrap().parent = None;
    tree.get_mut(&parent).unwrap().parent = Some(*id);

    tree.get_mut(id).unwrap().add_child(parent, None);
    tree.get_mut(&parent).unwrap().remove_child(id).unwrap();

    let edge = tree.get_mut(&parent).unwrap().parent_edge;
    tree.get_mut(&parent).unwrap().parent_edge = tree.get_mut(id).unwrap().parent_edge;
    tree.get_mut(id).unwrap().parent_edge = prev_edge;

    edge
}

/// Insert a new parent node for a pair of nodes
///
/// ```
/// use phylotree::tree::Tree;
///
/// // Simple case
/// let newick = "((A,B),C);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id_a = tree.get_by_name("A").unwrap().id;
/// let id_b = tree.get_by_name("B").unwrap().id;
///
/// nwr::insert_parent_pair(&mut tree, &id_a, &id_b);
///
/// assert_eq!(tree.to_newick().unwrap(), "(((A,B)),C);".to_string());
///
/// // Case with edge lengths
/// let newick = "((A:1,B:1):1,C:1);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id_a = tree.get_by_name("A").unwrap().id;
/// let id_b = tree.get_by_name("B").unwrap().id;
///
/// nwr::insert_parent_pair(&mut tree, &id_a, &id_b);
///
/// assert_eq!(tree.to_newick().unwrap(), "(((A:1,B:1)):1,C:1);".to_string());
/// ```
pub fn insert_parent_pair(tree: &mut Tree, id1: &NodeId, id2: &NodeId) -> NodeId {
    let old = tree.get_common_ancestor(id1, id2).unwrap();

    // Get original edge lengths
    let edge1 = tree.get(id1).unwrap().parent_edge;
    let edge2 = tree.get(id2).unwrap().parent_edge;

    // New node with parent (old) has no edge length
    let new = tree.add_child(Node::new(), old, None).unwrap();

    // Set children's new parent and edge length
    tree.get_mut(id1).unwrap().set_parent(new, edge1);
    tree.get_mut(&new).unwrap().add_child(*id1, edge1);
    tree.get_mut(&old).unwrap().remove_child(id1).unwrap();

    tree.get_mut(id2).unwrap().set_parent(new, edge2);
    tree.get_mut(&new).unwrap().add_child(*id2, edge2);
    tree.get_mut(&old).unwrap().remove_child(id2).unwrap();

    new
}

/// Removes a single node and connects its children to its parent
///
/// ```
/// use phylotree::tree::Tree;
///
/// // Simple case
/// let newick = "(A,(B)D);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id = tree.get_by_name("D").unwrap().id;
///
/// nwr::delete_node(&mut tree, &id).unwrap();
///
/// assert_eq!(tree.to_newick().unwrap(), "(A,B);".to_string());
///
/// // Case with edge lengths
/// let newick = "(A:1,(B:1)D:2);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id = tree.get_by_name("D").unwrap().id;
///
/// nwr::delete_node(&mut tree, &id).unwrap();
///
/// assert_eq!(tree.to_newick().unwrap(), "(A:1,B:3);".to_string());
/// ```
pub fn delete_node(tree: &mut Tree, id: &NodeId) -> anyhow::Result<()> {
    // Collect all necessary information first
    let parent;
    let to_remove;
    let children_info: Vec<(NodeId, Option<EdgeLength>, Option<EdgeLength>)>;
    {
        let node = tree.get(id)?;
        parent = node.parent.unwrap();
        to_remove = node.id;

        // Collect information for all child nodes
        children_info = node
            .children
            .iter()
            .map(|child| {
                let child_edge = node.get_child_edge(child);
                let parent_edge = node.parent_edge;
                (*child, child_edge, parent_edge)
            })
            .collect();
    }

    // Process all child nodes
    for (child, child_edge, parent_edge) in children_info {
        let new_edge = match (parent_edge, child_edge) {
            (Some(p), Some(c)) => Some(p + c),
            (Some(p), None) => Some(p),
            (None, Some(c)) => Some(c),
            (None, None) => None,
        };

        // Reset parent-child relationships
        tree.get_mut(&child)?.set_parent(parent, new_edge);
        tree.get_mut(&parent)?.add_child(child, new_edge);
    }

    // Remove the original node
    tree.get_mut(&parent)?.remove_child(&to_remove)?;

    Ok(())
}

/// Get the height of a node (maximum distance to its descendant leaves)
///
/// ```
/// use phylotree::tree::Tree;
///
/// // Simple case
/// let newick = "(A:1,(B:2)C:1);";
/// let mut tree = Tree::from_newick(newick).unwrap();
///
/// let id_c = tree.get_by_name("C").unwrap().id;
/// assert_eq!(nwr::node_height(&tree, &id_c), 2.0);
///
/// let id_b = tree.get_by_name("B").unwrap().id;
/// assert_eq!(nwr::node_height(&tree, &id_b), 0.0);
///
/// // Case with no edge lengths
/// let newick = "(A,(B)C);";
/// let mut tree = Tree::from_newick(newick).unwrap();
/// let id_c = tree.get_by_name("C").unwrap().id;
///
/// assert_eq!(nwr::node_height(&tree, &id_c), 0.0);
/// ```
pub fn node_height(tree: &Tree, id: &NodeId) -> EdgeLength {
    if tree.get(id).unwrap().is_tip() {
        return 0.0;
    }

    let mut heights = vec![];
    for child in tree.get_descendants(id).unwrap() {
        // eprintln!("child = {:#?}", child);

        if tree.get(&child).unwrap().is_tip() {
            if let Ok((Some(dist), _)) = tree.get_distance(id, &child) {
                heights.push(dist);
            }
        }
    }

    if heights.is_empty() {
        0.0
    } else {
        heights
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            .to_owned()
    }
}

// Named IDs that match the name rules
pub fn match_names(tree: &Tree, args: &clap::ArgMatches) -> BTreeSet<usize> {
    // IDs with names
    let id_of: BTreeMap<_, _> = get_name_id(tree);

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

    // Default is printing all named nodes
    let is_all = !(args.contains_id("node")
        || args.contains_id("file")
        || args.contains_id("regex"));

    if is_all {
        ids = id_of.values().cloned().collect();
    }

    // Include all descendants of internal nodes
    let is_descendants = if args.try_contains_id("descendants").is_ok() {
        args.get_flag("descendants")
    } else {
        false
    };

    if is_descendants {
        let nodes: Vec<Node> =
            ids.iter().map(|e| tree.get(e).unwrap().clone()).collect();
        for node in &nodes {
            if !node.is_tip() {
                for id in &tree.get_subtree(&node.id).unwrap() {
                    if tree.get(id).unwrap().name.is_some() {
                        ids.insert(*id);
                    }
                }
            }
        }
    }

    ids
}

// IDs that match the position rules
pub fn match_positions(tree: &Tree, args: &clap::ArgMatches) -> BTreeSet<usize> {
    let mut skip_internal = if args.try_contains_id("Internal").is_ok() {
        args.get_flag("Internal")
    } else {
        false
    };
    let skip_leaf = if args.try_contains_id("Leaf").is_ok() {
        args.get_flag("Leaf")
    } else {
        false
    };

    let is_monophyly = if args.try_contains_id("monophyly").is_ok() {
        args.get_flag("monophyly")
    } else {
        false
    };

    if is_monophyly {
        skip_internal = true;
    }

    // all matched IDs
    let mut ids = BTreeSet::new();

    // inorder needs IsBinary
    tree.preorder(&tree.get_root().unwrap())
        .unwrap()
        .iter()
        .for_each(|id| {
            let node = tree.get(id).unwrap();
            if node.is_tip() && !skip_leaf {
                ids.insert(*id);
            }
            if !node.is_tip() && !skip_internal {
                ids.insert(*id);
            }
        });

    ids
}

// Named IDs that belong to ancestors
pub fn match_restrict(tree: &Tree, args: &clap::ArgMatches) -> BTreeSet<usize> {
    // IDs with names
    let id_of: BTreeMap<_, _> = get_name_id(tree);

    // all matched IDs
    let mut ids = BTreeSet::new();

    if args.contains_id("term") {
        let nwrdir = if args.contains_id("dir") {
            std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
        } else {
            crate::nwr_path()
        };
        let conn = crate::connect_txdb(&nwrdir).unwrap();

        let mut tax_id_set = IntSpan::new();
        for term in args.get_many::<String>("term").unwrap() {
            let id = crate::term_to_tax_id(&conn, term).unwrap();
            let descendents: Vec<i32> = crate::get_all_descendent(&conn, id)
                .unwrap()
                .iter()
                .map(|n| *n as i32)
                .collect();
            tax_id_set.add_vec(descendents.as_ref());
        }

        let mode = args.get_one::<String>("mode").unwrap();
        let nodes: Vec<Node> = id_of
            .values()
            .map(|v| tree.get(v).unwrap().clone())
            .collect();
        for node in nodes.iter() {
            let term = match mode.as_str() {
                "label" => node.name.clone(),
                "taxid" => crate::get_comment_k(node, "T"),
                "species" => crate::get_comment_k(node, "S"),
                _ => unreachable!(),
            };

            if term.is_some() {
                let tax_id = match crate::term_to_tax_id(&conn, &term.unwrap()) {
                    Ok(id) => id,
                    Err(_) => continue,
                };
                if tax_id_set.contains(tax_id as i32) {
                    ids.insert(node.id);
                }
            }
        }
    } else {
        ids = id_of.values().cloned().collect();
    }

    ids
}

pub fn check_monophyly(tree: &Tree, ids: &BTreeSet<usize>) -> bool {
    let mut nodes: Vec<usize> = ids.iter().cloned().collect();
    if nodes.is_empty() {
        return false;
    }

    let mut sub_root = nodes.pop().unwrap();

    for id in &nodes {
        sub_root = tree.get_common_ancestor(&sub_root, id).unwrap();
    }

    let mut descendants = BTreeSet::new();
    for id in &tree.get_subtree(&sub_root).unwrap() {
        if tree.get(id).unwrap().is_tip() {
            descendants.insert(*id);
        }
    }

    descendants.eq(ids)
}

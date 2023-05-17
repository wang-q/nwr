use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub tax_id: i64,
    pub parent_tax_id: i64,
    pub rank: String,
    pub division: String,
    pub names: HashMap<String, Vec<String>>, // many synonym or common names
    pub comments: Option<String>,
    pub format_string: Option<String>,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(format_string) = &self.format_string {
            // Format the Node according to its format string.
            return write!(
                f,
                "{}",
                format_string
                    .replace("%taxid", &self.tax_id.to_string())
                    .replace("%name", &self.names.get("scientific name").unwrap()[0])
                    .replace("%rank", &self.rank)
            );
        }

        let mut lines = String::new();

        let sciname = &self.names.get("scientific name").unwrap()[0];
        let l1 = format!("{} - {}\n", sciname, self.rank);
        let l2 = std::iter::repeat("-")
            .take(l1.len() - 1)
            .collect::<String>();
        lines.push_str(&l1);
        lines.push_str(&l2);
        lines.push_str(&format!("\nNCBI Taxonomy ID: {}\n", self.tax_id));

        if self.names.contains_key("synonym") {
            lines.push_str("Same as:\n");
            for synonym in self.names.get("synonym").unwrap() {
                lines.push_str(&format!("* {}\n", synonym));
            }
        }

        if self.names.contains_key("genbank common name") {
            let genbank = &self.names.get("genbank common name").unwrap()[0];
            lines.push_str(&format!("Commonly named {}.\n", genbank));
        }

        if self.names.contains_key("common name") {
            lines.push_str("Also known as:\n");
            for name in self.names.get("common name").unwrap() {
                lines.push_str(&format!("* {}\n", name));
            }
        }

        if self.names.contains_key("authority") {
            lines.push_str("First description:\n");
            for authority in self.names.get("authority").unwrap() {
                lines.push_str(&format!("* {}\n", authority));
            }
        }

        lines.push_str(&format!("Part of the {}.\n", self.division));

        if let Some(ref comments) = self.comments {
            lines.push_str(&format!("\nComments: {}", comments));
        }

        write!(f, "{}\n", lines)
    }
}

/// A simple tree implementation
///
pub struct Tree {
    root: i64,
    pub nodes: HashMap<i64, Node>,
    pub children: HashMap<i64, HashSet<i64>>,
    marked: HashSet<i64>,
}

impl Tree {
    /// Create a new Tree containing the given nodes.
    pub fn new(root_id: i64, nodes: &[Node]) -> Tree {
        let mut tree = Tree {
            root: root_id,
            nodes: HashMap::new(),
            children: HashMap::new(),
            marked: HashSet::new(),
        };
        tree.add_nodes(nodes);
        tree
    }

    /// Add the given nodes to the Tree.
    pub fn add_nodes(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.nodes.entry(node.tax_id).or_insert({
                let mut node = node.clone();
                if node.format_string.is_none() {
                    node.format_string = Some(String::from("%rank: %name"));
                }
                node
            });

            if node.tax_id != node.parent_tax_id {
                self.children
                    .entry(node.parent_tax_id)
                    .and_modify(|children| {
                        children.insert(node.tax_id);
                    })
                    .or_insert({
                        let mut set = HashSet::new();
                        set.insert(node.tax_id);
                        set
                    });
            }
        }
    }

    /// Mark the nodes with this IDs.
    pub fn mark_nodes(&mut self, taxids: &[i64]) {
        for taxid in taxids.iter() {
            self.marked.insert(*taxid);
        }
    }

    /// Set the format string for all nodes.
    pub fn set_format_string(&mut self, format_string: String) {
        for node in self.nodes.values_mut() {
            node.format_string = Some(format_string.clone());
        }
    }

    /// Simplify the tree by removing all nodes that have only one child
    /// *and* are not marked.
    pub fn simplify(&mut self) {
        self.simplify_helper(self.root);
        self.children.retain(|_, v| !v.is_empty());
    }

    fn simplify_helper(&mut self, parent: i64) {
        let new_children = self.remove_single_child(parent);
        // TODO: remove that clone
        self.children.insert(parent, new_children.clone());
        // .unwrap() is safe here because new_children
        // is at least an empty set.
        for child in new_children.iter() {
            self.simplify_helper(*child);
        }
    }

    /// remove_single_child find the new children of a node by removing all
    /// unique child.
    fn remove_single_child(&self, parent: i64) -> HashSet<i64> {
        // nodes are the children of parent
        let mut new_children = HashSet::new();
        if let Some(nodes) = self.children.get(&parent) {
            for node in nodes.iter() {
                let mut node = node;
                while let Some(children) = self.children.get(node) {
                    if children.len() == 1 && !self.marked.contains(node) {
                        node = children.iter().next().unwrap();
                    } else {
                        break;
                    }
                }
                new_children.insert(*node);
            }
        }
        new_children
    }

    /// Helper function that actually makes the Newick format representation
    /// of the tree. The resulting String is in `n` and the current node is
    /// `taxid`.
    ///
    /// This function is recursive, hence it should be called only once with
    /// the root.
    fn newick_helper(&self, n: &mut String, taxid: i64) {
        // unwrap are safe here because of the way we build the tree
        // and the nodes.
        let node = self.nodes.get(&taxid).unwrap();

        if let Some(children) = self.children.get(&taxid) {
            n.push_str(&format!("({}", node)); // Mind the parenthesis
            n.push_str(",(");
            for child in children.iter() {
                self.newick_helper(n, *child);
                n.push(',');
            }

            // After iterating through the children, a comma left
            let _ = n.pop();
            // two closing parenthesis:
            // - one for the last child tree,
            // - another for the parent
            n.push_str("))");
        } else {
            n.push_str(&format!("{}", node)); // Mind the absent parenthesis
        }
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut n = String::new();

        if self.children.get(&self.root).unwrap().len() == 1 {
            let root = self.children.get(&1).unwrap().iter().next().unwrap();
            self.newick_helper(&mut n, *root);
        } else {
            self.newick_helper(&mut n, self.root);
        }
        n.push(';');

        write!(f, "{}", n)
    }
}

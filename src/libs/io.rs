use std::io::Read;
use std::{fmt, io, str};
use phylotree::tree::{Node, NodeId, Tree};

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

//----------------------------
// AsmEntry
//----------------------------
#[derive(Default, Clone)]
pub struct AsmEntry {
    name: String,
    vector: Vec<i32>,
}

impl AsmEntry {
    // Immutable accessors
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn vector(&self) -> &Vec<i32> {
        &self.vector
    }

    pub fn new() -> Self {
        Self {
            name: String::new(),
            vector: vec![],
        }
    }

    /// Constructed from range and seq
    ///
    /// ```
    /// # use nwr::AsmEntry;
    /// let name = "Es_coli_005008_GCF_013426115_1".to_string();
    /// let vector : Vec<i32> = vec![1,5,2,7,6,6];
    /// let entry = AsmEntry::from(&name, &vector);
    /// # assert_eq!(*entry.name(), "Es_coli_005008_GCF_013426115_1");
    /// # assert_eq!(*entry.vector().get(1).unwrap(), 5i32);
    /// ```
    pub fn from(name: &String, vector: &[i32]) -> Self {
        Self {
            name: name.clone(),
            vector: Vec::from(vector),
        }
    }

    /// ```
    /// # use nwr::AsmEntry;
    /// let line = "Es_coli_005008_GCF_013426115_1\t1,5,2,7,6,6".to_string();
    /// let entry = AsmEntry::parse(&line);
    /// # assert_eq!(*entry.name(), "Es_coli_005008_GCF_013426115_1");
    /// # assert_eq!(*entry.vector().get(1).unwrap(), 5i32);
    /// ```
    pub fn parse(line: &str) -> AsmEntry {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 2 {
            let name = fields[0].to_string();
            let parts: Vec<&str> = fields[1].split(',').collect();
            let vector: Vec<i32> = parts.iter().map(|e| e.parse::<i32>().unwrap()).collect();
            Self::from(&name, &vector)
        } else {
            Self::new()
        }
    }
}

impl fmt::Display for AsmEntry {
    /// To string
    ///
    /// ```
    /// # use nwr::AsmEntry;
    /// let name = "Es_coli_005008_GCF_013426115_1".to_string();
    /// let vector : Vec<i32> = vec![1,5,2,7,6,6];
    /// let entry = AsmEntry::from(&name, &vector);
    /// assert_eq!(entry.to_string(), "Es_coli_005008_GCF_013426115_1\t1,5,2,7,6,6\n");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\n",
            self.name(),
            self.vector.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(","),
        )?;
        Ok(())
    }
}



// https://www.maartengrootendorst.com/blog/distances/
// https://crates.io/crates/semanticsimilarity_rs

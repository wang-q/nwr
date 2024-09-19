use phylotree::tree::Tree;
use std::io::Read;
use std::{fmt, io, str};

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

pub fn parse_asm_line(line: &str) -> AsmEntry {

}


// https://www.maartengrootendorst.com/blog/distances/
// https://crates.io/crates/semanticsimilarity_rs

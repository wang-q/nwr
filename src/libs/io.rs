use phylotree::tree::Tree;
use std::io::Read;

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

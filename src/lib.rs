extern crate log;

mod libs;

pub use crate::libs::io::*;
pub use crate::libs::newick::*;
pub use crate::libs::taxonomy::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_functions() {
        // Test read_newick function
        let tree = read_newick("tests/newick/abc.nwk");
        assert!(tree.to_newick().is_ok());

        // Test format_tree function
        let newick = "(A,B);";
        let tree = phylotree::tree::Tree::from_newick(newick).unwrap();
        assert_eq!(format_tree(&tree, "  "), "(\n  A,\n  B\n);");
    }

    #[test]
    fn test_newick_functions() {
        let newick = "((A,B),C);";
        let mut tree = phylotree::tree::Tree::from_newick(newick).unwrap();

        // Test tree sorting functions
        order_tree_an(&mut tree, "an");
        assert!(tree.to_newick().is_ok());

        order_tree_nd(&mut tree, "nd");
        assert!(tree.to_newick().is_ok());

        // Test node name retrieval
        let names = get_names(&tree);
        assert!(names.contains(&"A".to_string()));
    }
}

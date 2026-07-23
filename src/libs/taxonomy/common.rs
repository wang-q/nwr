use crate::Taxon;

/// Add a taxon node to a phylogenetic tree.
///
/// Creates a `phylotree` node named after the taxon's scientific name (or
/// `"Unknown"` if absent) and attaches it as a child of `parent`, or as the
/// root when `parent` is `None`. Returns the new node id.
pub fn add_taxon(
    tree: &mut phylotree::tree::Tree,
    taxon: &Taxon,
    parent: Option<usize>,
) -> anyhow::Result<usize> {
    let mut node = phylotree::tree::Node::new();
    let name = taxon.scientific_name().unwrap_or("Unknown").to_string();
    node.set_name(name);

    let node_id = if let Some(p) = parent {
        tree.add_child(node, p, None)?
    } else {
        tree.add(node)
    };
    Ok(node_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_add_taxon_with_parent() {
        let mut tree = phylotree::tree::Tree::new();
        let root_node = phylotree::tree::Node::new();
        let root_id = tree.add(root_node);

        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            parent_tax_id: 1,
            names: HashMap::from([(
                "scientific name".to_string(),
                vec!["Test Phage".to_string()],
            )]),
            ..Default::default()
        };

        let node_id = add_taxon(&mut tree, &taxon, Some(root_id)).unwrap();
        assert_ne!(node_id, root_id);
    }

    #[test]
    fn test_add_taxon_without_parent() {
        let mut tree = phylotree::tree::Tree::new();

        let taxon = Taxon {
            tax_id: 1,
            rank: "no rank".to_string(),
            parent_tax_id: 1,
            names: HashMap::from([(
                "scientific name".to_string(),
                vec!["root".to_string()],
            )]),
            ..Default::default()
        };

        let node_id = add_taxon(&mut tree, &taxon, None).unwrap();
        assert_eq!(node_id, 0); // First node in tree
    }

    #[test]
    fn test_add_taxon_without_scientific_name() {
        let mut tree = phylotree::tree::Tree::new();

        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            parent_tax_id: 1,
            names: HashMap::new(),
            ..Default::default()
        };

        let node_id = add_taxon(&mut tree, &taxon, None).unwrap();
        // Should use "Unknown" as name
        let node = tree.get(&node_id).unwrap();
        assert_eq!(node.name.as_ref().unwrap(), "Unknown");
    }
}

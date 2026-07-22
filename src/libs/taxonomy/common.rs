use crate::Taxon;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

/// Parsed options for common-tree operations.
pub struct CommonOptions {
    /// Directory containing NCBI taxonomy databases.
    pub nwrdir: PathBuf,
    /// Taxonomy IDs or scientific names whose common tree should be output.
    pub terms: Vec<String>,
    /// Output file path.
    pub outfile: String,
}

/// Build and output the common phylogenetic tree for the given terms.
///
/// `terms` are NCBI taxonomy IDs or scientific names. They are resolved against
/// the taxonomy database in `nwrdir`, and the resulting Newick string is written
/// to `outfile` followed by a newline.
pub fn run(options: &CommonOptions) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(&options.outfile)?;
    let conn = crate::connect_txdb(&options.nwrdir)?;

    let mut tax_ids = vec![];
    for term in &options.terms {
        let id = crate::term_to_tax_id(&conn, term)?;
        tax_ids.push(id);
    }

    let mut tree = phylotree::tree::Tree::new();
    // tax_id to NodeID
    let mut id_of: HashMap<i64, usize> = HashMap::new();

    for tax_id in tax_ids {
        let lineage = crate::get_lineage(&conn, tax_id)?;

        for taxon in lineage.iter() {
            let cur_tax_id = taxon.tax_id;
            if !id_of.contains_key(&cur_tax_id) {
                let node_id = if cur_tax_id == 1 {
                    add_taxon(&mut tree, taxon, None)?
                } else {
                    let parent_tax_id = taxon.parent_tax_id;
                    let parent_id = id_of.get(&parent_tax_id).ok_or_else(|| {
                        anyhow::anyhow!("Parent ID not found: {}", parent_tax_id)
                    })?;
                    add_taxon(&mut tree, taxon, Some(*parent_id))?
                };
                id_of.insert(cur_tax_id, node_id);
            }
        }
    }

    tree.compress()?;
    let out_string = tree.to_newick()?;
    writer.write_all((out_string + "\n").as_ref())?;
    writer.flush()?;

    Ok(())
}

fn add_taxon(
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

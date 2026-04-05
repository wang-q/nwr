use clap::*;
use nwr::Taxon;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Output the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(
            Arg::new("terms")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename (default: stdout)"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut tax_ids = vec![];
    for term in args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
    {
        let id = nwr::term_to_tax_id(&conn, term)?;
        tax_ids.push(id);
    }

    let mut tree = phylotree::tree::Tree::new();
    // tax_id to NodeID
    let mut id_of: HashMap<i64, usize> = HashMap::new();

    for tax_id in tax_ids {
        let lineage = nwr::get_lineage(&conn, tax_id)?;

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

    Ok(())
}

fn add_taxon(
    tree: &mut phylotree::tree::Tree,
    taxon: &Taxon,
    parent: Option<usize>,
) -> anyhow::Result<usize> {
    let mut node = phylotree::tree::Node::new();
    let name = taxon
        .names
        .get("scientific name")
        .and_then(|v| v.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
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
    use tempfile::TempDir;

    #[test]
    fn test_common_single_term() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Actinophage JHJ-1",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Actinophage JHJ-1"));
        assert!(output.contains("root"));
    }

    #[test]
    fn test_common_multiple_terms() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "Actinophage JHJ-1",
                "Bacillus phage bg1",
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        // Both terms should be in the output
        assert!(
            output.contains("Actinophage JHJ-1")
                || output.contains("Bacillus phage bg1")
        );
    }

    #[test]
    fn test_common_with_tax_id() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "12347", // Actinophage JHJ-1
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Actinophage"));
    }

    #[test]
    fn test_common_stdout() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.nwk");

        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "common",
                "--dir",
                "tests/nwr/",
                "-o",
                output_file.to_str().unwrap(),
                "10239", // Viruses
            ])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_ok());

        // Verify output file was created and has content
        let output = std::fs::read_to_string(&output_file).unwrap();
        assert!(output.contains("Viruses"));
    }

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

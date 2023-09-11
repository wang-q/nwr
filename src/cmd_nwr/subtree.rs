use clap::*;
use std::collections::{BTreeMap, BTreeSet};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("subtree")
        .about("Extract a subtree")
        .after_help(
            r###"
Output a subtree (clade) rooted at the lowest common ancestor of all nodes passed in

* `--regex` is case insensitive
* `--monophyly` means the subtree should only contains the nodes passed in
    * It will check terminal nodes (with names) against the ones provided
    * If you provide a named internal node, its descendants will not automatically be included
    * Nodes with the same name CAN cause problems
"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .num_args(1)
                .index(1)
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("node")
                .long("node")
                .short('n')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Node name"),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .num_args(1)
                .help("A file contains node names"),
        )
        .arg(
            Arg::new("regex")
                .long("regex")
                .short('r')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Nodes match the regular expression"),
        )
        .arg(
            Arg::new("monophyly")
                .long("monophyly")
                .short('m')
                .action(ArgAction::SetTrue)
                .help("Only print the subtree when it's monophyletic"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let is_monophyly = args.get_flag("monophyly");

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = nwr::read_newick(infile);

    // ids with names
    let id_of: BTreeMap<_, _> = nwr::get_name_id(&tree);

    // All IDs matched
    let ids = nwr::match_names(&tree, args);

    if !ids.is_empty() {
        let mut nodes: Vec<usize> = ids.iter().cloned().collect();
        let mut sub_root = nodes.pop().unwrap();

        for id in &nodes {
            sub_root = tree.get_common_ancestor(&sub_root, id).unwrap();
        }

        if is_monophyly {
            let name_of: BTreeMap<usize, String> =
                id_of.iter().map(|(k, v)| (v.clone(), k.clone())).collect();

            let mut descendants = BTreeSet::new();
            for id in &tree.get_subtree(&sub_root).unwrap() {
                if name_of.contains_key(id) {
                    if tree.get(id).unwrap().is_tip() {
                        descendants.insert(id.clone());
                    }
                }
            }

            if ids.ne(&descendants) {
                return Ok(());
            }
        }

        let out_string = nwr::format_subtree(&tree, &sub_root, "") + ";";
        writer.write_all((out_string + "\n").as_ref())?;
    }

    Ok(())
}

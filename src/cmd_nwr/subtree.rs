use clap::*;
use regex::RegexBuilder;
use std::collections::{HashMap, HashSet};

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
    * Nodes with the same name can cause problems
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
    let id_of: HashMap<_, _> = nwr::get_name_id(&tree);

    // all IDs to be modified
    let mut ids = HashSet::new();

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
        let file = args.get_one::<String>("node").unwrap();
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

    // eprintln!("ids = {:#?}", ids);

    if !ids.is_empty() {
        let mut nodes: Vec<usize> = ids.iter().cloned().collect();
        let mut sub_root = nodes.pop().unwrap();

        for id in &nodes {
            sub_root = tree.get_common_ancestor(&sub_root, id).unwrap();
        }

        if is_monophyly {
            let name_of: HashMap<usize, String> = id_of.iter()
                .map(|(k, v)| (v.clone(), k.clone())).collect();

            let mut descendants = HashSet::new();
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
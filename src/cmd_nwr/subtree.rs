use clap::*;
use phylotree::tree::Node;
use std::collections::{BTreeMap, BTreeSet};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("subtree")
        .about("Extract a subtree")
        .after_help(
            r###"
Output a subtree (clade) rooted at the lowest common ancestor of all nodes passed in

* Match names
    * The intersection between the nodes in the tree and the provided
    * Nodes matching the case insensitive regular expression(s)
    * Prints all named nodes if none of `-n`, `-f` and `-r` are set.
* Match lineage
    * Like `nwr restrict`, print descendants of the provided terms
      in the form of a Taxonomy ID or scientific name
    * `--mode` - Taxonomy terms in label, taxid (:T=), or species (:S=)
* Match monophyly
    * `--monophyly` means the subtree should only contains the nodes passed in
    * It will check terminal nodes (with names) against the ones provided
    * With `-D`, a named internal node's descendants will automatically be included
    * Nodes with the same name CAN cause problems
    * Activate `-I`

* `--condense` - Instead of outputting the subtree, condense the subtree with the name provided

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
            Arg::new("descendants")
                .long("descendants")
                .short('D')
                .action(ArgAction::SetTrue)
                .help("Include all descendants of internal nodes"),
        )
        .arg(
            Arg::new("term")
                .long("term")
                .short('t')
                .num_args(1)
                .action(ArgAction::Append)
                .help("The ancestor(s)"),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Change working directory"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("label"),
                    builder::PossibleValue::new("taxid"),
                    builder::PossibleValue::new("species"),
                ])
                .default_value("label")
                .help("Where we can find taxonomy terms"),
        )
        .arg(
            Arg::new("monophyly")
                .long("monophyly")
                .short('M')
                .action(ArgAction::SetTrue)
                .help("Only print the subtree when it's monophyletic"),
        )
        .arg(
            Arg::new("condense")
                .long("condense")
                .short('c')
                .num_args(1)
                .help("The name of the condensed node"),
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
    //----------------------------
    // Args
    //----------------------------
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let is_monophyly = args.get_flag("monophyly");
    let is_condense = args.contains_id("condense");

    let infile = args.get_one::<String>("infile").unwrap();
    let mut tree = nwr::read_newick(infile);

    // ids with names
    let id_of: BTreeMap<_, _> = nwr::get_name_id(&tree);

    // All IDs matching names
    let ids_name = nwr::match_names(&tree, args);

    // lineage restrict
    let ids_restrict = nwr::match_restrict(&tree, args);

    // All IDs matched
    let ids: BTreeSet<usize> = ids_name.intersection(&ids_restrict).cloned().collect();

    if !ids.is_empty() {
        let mut nodes: Vec<usize> = ids.iter().cloned().collect();
        let mut sub_root_id = nodes.pop().unwrap();

        for id in &nodes {
            sub_root_id = tree.get_common_ancestor(&sub_root_id, id).unwrap();
        }

        if is_monophyly {
            let name_of: BTreeMap<usize, String> =
                id_of.iter().map(|(k, v)| (*v, k.clone())).collect();

            let mut descendants = BTreeSet::new();
            for id in &tree.get_subtree(&sub_root_id).unwrap() {
                if name_of.contains_key(id) && tree.get(id).unwrap().is_tip() {
                    descendants.insert(*id);
                }
            }

            if ids.ne(&descendants) {
                if is_condense {
                    let out_string = nwr::format_tree(&tree, "");
                    writer.write_all((out_string + "\n").as_ref())?;
                }
                return Ok(());
            }
        }

        let out_string = if is_condense {
            // parent of current sub_root
            let parent_id = tree.get(&sub_root_id).unwrap().parent.unwrap();
            let sub_root = tree.get(&sub_root_id).unwrap();

            // create a new node
            let condense = args.get_one::<String>("condense").unwrap();
            let mut new_node = Node::new_named(condense);

            // old comment may contains taxonomy terms
            // new_node.comment = sub_root.comment.clone();

            nwr::add_comment_kv(&mut new_node, "member", ids.len().to_string().as_str());
            nwr::add_comment_kv(&mut new_node, "tri", "white");
            let edge = sub_root.parent_edge;

            // remove sub_root
            tree.prune(&sub_root_id)?;
            tree.add_child(new_node, parent_id, edge)?;

            nwr::format_tree(&tree, "")
        } else {
            nwr::format_subtree(&tree, &sub_root_id, "") + ";"
        };
        writer.write_all((out_string + "\n").as_ref())?;
    }

    Ok(())
}

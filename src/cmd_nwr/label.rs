use clap::*;
use std::collections::BTreeSet;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("label")
        .about("Labels in the Newick file")
        .after_help(
            r###"
This tool selectively outputs the names of the nodes in the tree

* Match positions
    * `-I`, `-L`
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

* Set `--column` will output a TSV file with addtional columns
    * `dup` - duplicate the node name
    * `taxid` - `:T=` field in comment
    * `species` - `:S=` field in comment
    * `full` - full comment

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
            Arg::new("Internal")
                .long("Internal")
                .short('I')
                .action(ArgAction::SetTrue)
                .help("Don't print internal labels"),
        )
        .arg(
            Arg::new("Leaf")
                .long("Leaf")
                .short('L')
                .action(ArgAction::SetTrue)
                .help("Don't print leaf labels"),
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
                .help("Only print the labels when they form a monophyletic subtree"),
        )
        .arg(
            Arg::new("column")
                .long("column")
                .short('c')
                .action(ArgAction::Append)
                .value_parser([
                    builder::PossibleValue::new("dup"),
                    builder::PossibleValue::new("taxid"),
                    builder::PossibleValue::new("species"),
                    builder::PossibleValue::new("full"),
                ])
                .help("Where we can find taxonomy terms"),
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

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = nwr::read_newick(infile);

    let is_monophyly = args.get_flag("monophyly");

    let mut columns = vec![];
    if args.contains_id("column") {
        for column in args.get_many::<String>("column").unwrap() {
            columns.push(column.to_string());
        }
    }

    //----------------------------
    // Operating
    //----------------------------
    // All IDs matching positions
    let ids_pos = nwr::match_positions(&tree, args);

    // All IDs matching names
    let ids_name = nwr::match_names(&tree, args);

    // lineage restrict
    let ids_restrict = nwr::match_restrict(&tree, args);

    let mut ids: BTreeSet<usize> = ids_pos.intersection(&ids_name).cloned().collect();
    ids = ids.intersection(&ids_restrict).cloned().collect();

    // Print nothing if check_monophyly() failed
    if is_monophyly && !nwr::check_monophyly(&tree, &ids) {
        return Ok(());
    }

    //----------------------------
    // Output
    //----------------------------
    for id in ids.iter() {
        let node = tree.get(id).unwrap();
        if let Some(x) = node.name.clone() {
            let mut out_string = format!("{}", x);
            if !columns.is_empty() {
                for column in columns.iter() {
                    match column.as_str() {
                        "dup" => out_string += format!("\t{}", x).as_str(),
                        "taxid" => {
                            out_string += format!(
                                "\t{}",
                                nwr::get_comment_k(&node, "T").unwrap_or("".to_string())
                            )
                            .as_str()
                        }
                        "species" => {
                            out_string += format!(
                                "\t{}",
                                nwr::get_comment_k(&node, "S").unwrap_or("".to_string())
                            )
                            .as_str()
                        }
                        "full" => {
                            out_string += format!(
                                "\t{}",
                                node.comment.clone().unwrap_or("".to_string())
                            )
                            .as_str()
                        }
                        _ => unreachable!(),
                    }
                }
            }

            writer.write_fmt(format_args!("{}\n", out_string)).unwrap();
        }
    }

    Ok(())
}

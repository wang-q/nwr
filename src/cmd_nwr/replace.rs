use clap::*;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("replace")
        .about("Replace node names/comments in a Newick file")
        .after_help(
            r###"
* <replace.tsv> is a tab-separated file containing two or more fields

    original_name   replace more_replace

    * If you want to remove the name, set the second field to an empty string
    * From the third field and onwards, it will be inserted into the comment section as is

* `--mode` - label, taxid (:T=), species (:S=), and asis

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
            Arg::new("replace.tsv")
                .required(true)
                .num_args(1..)
                .index(2)
                .help("Path to replace.tsv"),
        )
        .arg(
            Arg::new("Internal")
                .long("Internal")
                .short('I')
                .action(ArgAction::SetTrue)
                .help("Skip internal labels"),
        )
        .arg(
            Arg::new("Leaf")
                .long("Leaf")
                .short('L')
                .action(ArgAction::SetTrue)
                .help("Skip leaf labels"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("label"),
                    builder::PossibleValue::new("taxid"),
                    builder::PossibleValue::new("species"),
                    builder::PossibleValue::new("asis"),
                ])
                .default_value("label")
                .help("Where we place the replaces"),
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
    let mut tree = nwr::read_newick(infile);
    let mode = args.get_one::<String>("mode").unwrap();

    // ids with names
    let id_of: BTreeMap<_, _> = nwr::get_name_id(&tree);
    // All IDs matching positions
    let ids_pos = nwr::match_positions(&tree, args);

    let mut replace_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for rfile in args.get_many::<String>("replace.tsv").unwrap() {
        for line in intspan::read_lines(rfile) {
            let parts: Vec<_> = line.split('\t').collect();

            if parts.is_empty() || parts.len() == 1 {
                continue;
            } else {
                let name = parts.first().unwrap().to_string();
                let replaces = parts
                    .iter()
                    .skip(1)
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>();
                replace_of.insert(name.to_string(), replaces);
            }
        }
    }

    for (k, id) in id_of.iter() {
        if replace_of.contains_key(k) && ids_pos.contains(id) {
            let node = tree.get_mut(id).unwrap();

            let replaces = replace_of.get(k).unwrap();

            let first = replaces.first().unwrap().to_string();
            match mode.as_str() {
                "label" => node.set_name(first.clone()),
                "taxid" => nwr::add_comment_kv(node, "T", &first),
                "species" => nwr::add_comment_kv(node, "S", &first),
                "asis" => nwr::add_comment(node, &first),
                _ => unreachable!(),
            }

            replaces
                .iter()
                .skip(1)
                .for_each(|e| nwr::add_comment(node, e));
        }
    }

    let out_string = nwr::format_tree(&tree, "");
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

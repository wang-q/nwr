use clap::*;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("comment")
        .about("Add comments to node(s) in a Newick file")
        .after_help(
            r###"
* Comments are in the NHX-like format
    * :key=value

* For nodes with names, set `--node` to the name
* For nodes without names (e.g., internal nodes), set `--lca` to a combination
  of two node names, separated by commas
    * `--lca A,B`

* Set `--string` to add free-form strings

* The following options are used for visualization
    * `--color`, `--label` and `--comment` take 1 argument
    * `--dot`, `--bar` and `--rec` take 1 or 0 argument

* Predefined colors for `--color`, `--dot` and `--bar`
    * {red}{RGB}{188,36,46}
    * {black}{RGB}{26,25,25}
    * {grey}{RGB}{129,130,132}
    * {green}{RGB}{32,128,108}
    * {purple}{RGB}{160,90,150}
* Colors for background rectangles `--rec`
    * {LemonChiffon}{RGB}{251, 248, 204}
    * {ChampagnePink}{RGB}{253, 228, 207}
    * {TeaRose}{RGB}{255, 207, 210}
    * {PinkLavender}{RGB}{241, 192, 232}
    * {Mauve}{RGB}{207, 186, 240}
    * {JordyBlue}{RGB}{163, 196, 243}
    * {NonPhotoBlue}{RGB}{144, 219, 244}
    * {ElectricBlue}{RGB}{142, 236, 245}
    * {Aquamarine}{RGB}{152, 245, 225}
    * {Celadon}{RGB}{185, 251, 192}

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
            Arg::new("lca")
                .long("lca")
                .short('l')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Lowest common ancestor of two nodes"),
        )
        .arg(
            Arg::new("string")
                .long("string")
                .short('s')
                .num_args(1)
                .help("Free-form strings"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .num_args(1)
                .help("Color of names"),
        )
        .arg(
            Arg::new("label")
                .long("label")
                .num_args(1)
                .help("Add this label to the south west of the node"),
        )
        .arg(
            Arg::new("comment")
                .long("comment")
                .num_args(1)
                .help("comment text after names"),
        )
        .arg(
            Arg::new("dot")
                .long("dot")
                .num_args(0..=1)
                .default_missing_value("black")
                .help("Place a dot in the node; value as color"),
        )
        .arg(
            Arg::new("bar")
                .long("bar")
                .num_args(0..=1)
                .default_missing_value("black")
                .help("Place a bar in the middle of the parent edge; value as color"),
        )
        .arg(
            Arg::new("rec")
                .long("rec")
                .num_args(0..=1)
                .default_missing_value("LemonChiffon")
                .help(
                    "Place a rectangle in the background of the subtree; value as color",
                ),
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

    let string = args.get_one::<String>("string");

    let label = args.get_one::<String>("label");
    let color = args.get_one::<String>("color");
    let comment = args.get_one::<String>("comment");

    let dot = args.get_one::<String>("dot");
    let bar = args.get_one::<String>("bar");
    let rec = args.get_one::<String>("rec");

    let infile = args.get_one::<String>("infile").unwrap();
    let mut tree = nwr::read_newick(infile);

    // ids with names
    let id_of: BTreeMap<_, _> = nwr::get_name_id(&tree);

    // all IDs to be modified
    let mut ids = vec![];

    // ids supplied by --node
    if args.contains_id("node") {
        for name in args.get_many::<String>("node").unwrap() {
            if id_of.contains_key(name) {
                let id = id_of.get(name).unwrap();
                ids.push(*id);
            }
        }
    }

    // ids supplied by --lca
    if args.contains_id("lca") {
        for lca in args.get_many::<String>("lca").unwrap() {
            let parts = lca.split(',').map(|e| e.to_string()).collect::<Vec<_>>();
            if parts.len() != 2 {
                continue;
            }

            if parts.iter().all(|e| id_of.contains_key(e)) {
                let id1 = id_of.get(parts.first().unwrap()).unwrap();
                let id2 = id_of.get(parts.last().unwrap()).unwrap();

                let id = tree.get_common_ancestor(id1, id2);

                if let Ok(x) = id {
                    ids.push(x);
                }
            }
        }
    }

    for id in &ids {
        let node = tree.get_mut(id).unwrap();

        if let Some(x) = string {
            nwr::add_comment(node, x);
        }

        if let Some(x) = label {
            nwr::add_comment_kv(node, "label", x);
        }
        if let Some(x) = color {
            nwr::add_comment_kv(node, "color", x);
        }
        if let Some(x) = comment {
            nwr::add_comment_kv(node, "comment", x);
        }

        if let Some(x) = dot {
            nwr::add_comment_kv(node, "dot", x);
        }
        if let Some(x) = bar {
            nwr::add_comment_kv(node, "bar", x);
        }
        if let Some(x) = rec {
            nwr::add_comment_kv(node, "rec", x);
        }
    }

    let out_string = nwr::format_tree(&tree, "");
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

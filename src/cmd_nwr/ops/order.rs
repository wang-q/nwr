use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("order")
        .about("Order nodes in a Newick file")
        .after_help(
            r###"
* Traverse the entire tree in a breadth-first order
* Sort the children of each node without changing the topology

* `--an` and `--nd` can be enabled at the same time, sorted first by `--an` and then by `--nd`
* `--list` will be processed before `--an` and `--nd`

* Sort orders:
    * `--list`: By a list of names in the file, one name per line
    * `--an/--anr`: By alphanumeric order of labels
    * `--nd/--ndr`: By number of descendants

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .num_args(1)
                .index(1)
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(arg!(--nd  "By number of descendants"))
        .arg(arg!(--ndr "By number of descendants, reversely"))
        .group(ArgGroup::new("number-of-descendants").args(["nd", "ndr"]))
        .arg(arg!(--an  "By alphanumeric order of labels"))
        .arg(arg!(--anr "By alphanumeric order of labels, reversely"))
        .group(ArgGroup::new("alphanumeric").args(["an", "anr"]))
        .arg(
            Arg::new("list")
                .long("list")
                .short('l')
                .num_args(1)
                .help("Order by a list of names in the file"),
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

    let opt_nd = match args.get_one::<Id>("number-of-descendants") {
        None => "",
        Some(x) => x.as_str(),
    };
    let opt_an = match args.get_one::<Id>("alphanumeric") {
        None => "",
        Some(x) => x.as_str(),
    };

    let infile = args.get_one::<String>("infile").unwrap();
    let mut tree = nwr::read_newick(infile);

    if args.contains_id("list") {
        let list_file = args.get_one::<String>("list").unwrap();
        let names: Vec<String> = intspan::read_first_column(list_file);
        nwr::order_tree_list(&mut tree, &names);
    }
    if !opt_an.is_empty() {
        nwr::order_tree_an(&mut tree, opt_an);
    }
    if !opt_nd.is_empty() {
        nwr::order_tree_nd(&mut tree, opt_nd);
    }

    // eprintln!("tree = {:#?}", tree.to_newick());
    let out_string = tree.to_newick().unwrap();
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

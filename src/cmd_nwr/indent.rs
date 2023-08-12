use clap::*;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("indent")
        .about("Indent the Newick file")
        .after_help(
            r###"
* Set `--text` to something other than whitespaces will result in an invalid Newick file
    * Use `--text ".   "` can produce visual guide lines

* Set `--text` to empty ("") will remove indentation

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
            Arg::new("text")
                .long("text")
                .short('t')
                .num_args(1)
                .default_value("  ")
                .help("Use this text instead of the default two spaces"),
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

    let text = args.get_one::<String>("text").unwrap();

    let infile = args.get_one::<String>("infile").unwrap();
    let tree = nwr::read_newick(infile);

    let out_string = nwr::format_tree(&tree, text);
    writer.write_all((out_string + "\n").as_ref())?;

    Ok(())
}

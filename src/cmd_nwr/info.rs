use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("info")
        .about("Information of Taxonomy ID(s) or scientific name(s)")
        .after_help(include_str!("../../docs/help/info.md"))
        .arg(
            Arg::new("terms")
                .help("Taxonomy ID(s) or scientific name(s)")
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
            Arg::new("tsv")
                .long("tsv")
                .action(ArgAction::SetTrue)
                .help("Output the results as TSV"),
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

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    nwr::libs::taxonomy::info::run(&nwr::libs::taxonomy::info::InfoOptions {
        nwrdir: nwr::get_nwr_dir(args, "dir")?,
        terms: args
            .get_many::<String>("terms")
            .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
            .cloned()
            .collect(),
        is_tsv: args.get_flag("tsv"),
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

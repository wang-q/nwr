use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("lineage")
        .about("Output the lineage of the term")
        .after_help(include_str!("../../docs/help/lineage.md"))
        .arg(
            Arg::new("term")
                .help("The NCBI Taxonomy ID or scientific name")
                .required(true)
                .num_args(1)
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

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    nwr::libs::taxonomy::lineage::run(&nwr::libs::taxonomy::lineage::LineageOptions {
        nwrdir: nwr::get_nwr_dir(args, "dir")?,
        term: args
            .get_one::<String>("term")
            .ok_or_else(|| anyhow::anyhow!("No term provided"))?
            .clone(),
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

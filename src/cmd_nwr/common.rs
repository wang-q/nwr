use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Output the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(
            Arg::new("terms")
                .help("The NCBI Taxonomy ID or scientific name")
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
    nwr::libs::taxonomy::common::run(&nwr::libs::taxonomy::common::CommonOptions {
        nwrdir: nwr::get_nwr_dir(args, "dir")?,
        terms: args
            .get_many::<String>("terms")
            .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
            .cloned()
            .collect(),
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

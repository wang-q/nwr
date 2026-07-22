use super::args;
use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("info")
        .about("Shows information of Taxonomy ID(s) or scientific name(s)")
        .after_help(include_str!("../../docs/help/info.md"))
        .arg(args::terms_arg("Taxonomy ID(s) or scientific name(s)"))
        .arg(args::dir_arg())
        .arg(
            Arg::new("tsv")
                .long("tsv")
                .action(ArgAction::SetTrue)
                .help("Output the results as TSV"),
        )
        .arg(args::outfile_arg())
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

use super::args;
use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("common")
        .about("Outputs the common tree of terms")
        .after_help(include_str!("../../docs/help/common.md"))
        .arg(args::terms_arg("The NCBI Taxonomy ID or scientific name"))
        .arg(args::dir_arg())
        .arg(args::outfile_arg())
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

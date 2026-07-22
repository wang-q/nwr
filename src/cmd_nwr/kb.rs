use super::args;
use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("kb")
        .about("Extracts bundled knowledge-base archives")
        .after_help(include_str!("../../docs/help/kb.md"))
        .arg(
            Arg::new("infile")
                .help("Document to print (bac120, ar53)")
                .num_args(1)
                .required(true)
                .index(1),
        )
        .arg(args::outdir_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    nwr::libs::kb::run(&nwr::libs::kb::KbOptions {
        infile: args.get_one::<String>("infile").unwrap().clone(),
        outdir: args.get_one::<String>("outdir").unwrap().clone(),
    })
}

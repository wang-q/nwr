use clap::*;

pub mod indent;
pub mod comment;
pub mod tex;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("viz")
        .about("Newick visualization commands")
        .subcommand_required(true)
        .subcommand(indent::make_subcommand())
        .subcommand(comment::make_subcommand())
        .subcommand(tex::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("indent", sub_args)) => indent::execute(sub_args),
        Some(("comment", sub_args)) => comment::execute(sub_args),
        Some(("tex", sub_args)) => tex::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

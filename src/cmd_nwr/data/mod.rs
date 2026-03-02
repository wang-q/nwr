use clap::*;

pub mod label;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("data")
        .about("Newick data commands")
        .subcommand_required(true)
        .subcommand(label::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("label", sub_args)) => label::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

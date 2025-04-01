use clap::*;

pub mod distance;
pub mod label;
pub mod stat;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("data")
        .about("Newick data commands")
        .subcommand_required(true)
        .subcommand(label::make_subcommand())
        .subcommand(stat::make_subcommand())
        .subcommand(distance::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("label", sub_args)) => label::execute(sub_args),
        Some(("stat", sub_args)) => stat::execute(sub_args),
        Some(("distance", sub_args)) => distance::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

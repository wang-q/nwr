use clap::*;

pub mod compare;
pub mod format;
pub mod pair;
pub mod phylip;
pub mod subset;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("mat")
        .about("Distance matrix commands")
        .subcommand_required(true)
        .subcommand(compare::make_subcommand())
        .subcommand(format::make_subcommand())
        .subcommand(pair::make_subcommand())
        .subcommand(phylip::make_subcommand())
        .subcommand(subset::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("compare", sub_args)) => compare::execute(sub_args),
        Some(("format", sub_args)) => format::execute(sub_args),
        Some(("pair", sub_args)) => pair::execute(sub_args),
        Some(("phylip", sub_args)) => phylip::execute(sub_args),
        Some(("subset", sub_args)) => subset::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

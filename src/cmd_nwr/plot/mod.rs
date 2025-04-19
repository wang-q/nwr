use clap::*;

pub mod hh;
pub mod venn;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("plot")
        .about("Plot commands")
        .subcommand_required(true)
        .subcommand(hh::make_subcommand())
        .subcommand(venn::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("hh", sub_args)) => hh::execute(sub_args),
        Some(("venn", sub_args)) => venn::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

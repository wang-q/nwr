use clap::*;

pub mod upgma;
pub mod nj;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("build")
        .about("Build tree from distance matrix")
        .subcommand_required(true)
        .subcommand(upgma::make_subcommand())
        .subcommand(nj::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("upgma", sub_args)) => upgma::execute(sub_args),
        Some(("nj", sub_args)) => nj::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

use clap::*;

pub mod order;
pub mod prune;
pub mod rename;
pub mod replace;
pub mod reroot;
pub mod subtree;
pub mod topo;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("ops")
        .about("Newick operation commands")
        .subcommand_required(true)
        .subcommand(order::make_subcommand())
        .subcommand(rename::make_subcommand())
        .subcommand(replace::make_subcommand())
        .subcommand(subtree::make_subcommand())
        .subcommand(topo::make_subcommand())
        .subcommand(prune::make_subcommand())
        .subcommand(reroot::make_subcommand())
}

/// Execute pkg command
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    match args.subcommand() {
        Some(("order", sub_args)) => order::execute(sub_args),
        Some(("rename", sub_args)) => rename::execute(sub_args),
        Some(("replace", sub_args)) => replace::execute(sub_args),
        Some(("subtree", sub_args)) => subtree::execute(sub_args),
        Some(("topo", sub_args)) => topo::execute(sub_args),
        Some(("prune", sub_args)) => prune::execute(sub_args),
        Some(("reroot", sub_args)) => reroot::execute(sub_args),
        _ => unreachable!(
            "Exhausted list of subcommands and subcommand_required prevents `None`"
        ),
    }
}

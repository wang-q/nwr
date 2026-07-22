use super::args;
use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("member")
        .about("Lists members (of certain ranks) under ancestral term(s)")
        .after_help(include_str!("../../docs/help/member.md"))
        .arg(
            Arg::new("terms")
                .help("The ancestor(s)")
                .required(true)
                .num_args(1..)
                .index(1),
        )
        .arg(args::dir_arg())
        .arg(args::rank_arg())
        .arg(
            Arg::new("env")
                .long("env")
                .action(ArgAction::SetTrue)
                .help("Include division `Environmental samples`"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let ranks: Vec<String> = args
        .get_many::<String>("rank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    nwr::libs::taxonomy::member::run(&nwr::libs::taxonomy::member::MemberOptions {
        nwrdir,
        terms,
        ranks,
        is_env: args.get_flag("env"),
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("member")
        .about("List members (of certain ranks) under ancestral term(s)")
        .after_help(include_str!("../../docs/help/member.md"))
        .arg(
            Arg::new("terms")
                .help("The ancestor(s)")
                .required(true)
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Specify the NWR data directory"),
        )
        .arg(
            Arg::new("rank")
                .long("rank")
                .short('r')
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Taxonomic rank(s) to list"),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .action(ArgAction::SetTrue)
                .help("Include division `Environmental samples`"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename (default: stdout)"),
        )
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.to_string());
        }
    }

    nwr::libs::taxonomy::member::run(&nwr::libs::taxonomy::member::MemberOptions {
        nwrdir,
        terms,
        ranks,
        is_env: args.get_flag("env"),
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

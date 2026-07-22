use super::args;
use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("append")
        .about("Appends taxonomic rank fields to a TSV file")
        .after_help(include_str!("../../docs/help/append.md"))
        .arg(
            Arg::new("infiles")
                .required(true)
                .num_args(1..)
                .index(1)
                .help("Input TSV file(s) to process. Use 'stdin' for standard input"),
        )
        .arg(args::dir_arg())
        .arg(args::rank_arg())
        .arg(args::column_arg())
        .arg(
            Arg::new("id")
                .long("id")
                .action(ArgAction::SetTrue)
                .help("Also append taxon IDs for each rank"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let column: usize = *args.get_one("column").unwrap();

    let ranks: Vec<String> = args
        .get_many::<String>("rank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let infiles: Vec<String> = args
        .get_many::<String>("infiles")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    nwr::libs::taxonomy::append::run(&nwr::libs::taxonomy::append::AppendOptions {
        nwrdir,
        infiles,
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
        column,
        ranks,
        is_id: args.get_flag("id"),
    })
}

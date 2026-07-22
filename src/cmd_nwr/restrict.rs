use super::args;
use clap::*;
use simplelog::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("restrict")
        .about("Restricts taxonomy terms to ancestral descendants")
        .after_help(include_str!("../../docs/help/restrict.md"))
        .arg(args::terms_arg("The ancestor(s)"))
        .arg(args::dir_arg())
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .num_args(1..)
                .action(ArgAction::Append)
                .default_value("stdin")
                .help("Input filename. 'stdin' for standard input"),
        )
        .arg(args::column_arg())
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("exclude lines matching terms"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let column: usize = *args.get_one("column").unwrap();
    let is_exclude = args.get_flag("exclude");

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let files: Vec<String> = args
        .get_many::<String>("file")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    nwr::libs::taxonomy::restrict::run(&nwr::libs::taxonomy::restrict::RestrictOptions {
        nwrdir,
        terms,
        files,
        column,
        is_exclude,
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
    })
}

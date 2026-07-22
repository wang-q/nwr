use super::args;
use clap::*;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("abbr")
        .about("Abbreviates strain scientific names")
        .after_help(include_str!("../../docs/help/abbr.md"))
        .arg(
            Arg::new("infile")
                .help("Input file to process. Use 'stdin' for standard input")
                .num_args(1)
                .default_value("stdin")
                .index(1),
        )
        .arg(
            Arg::new("column")
                .long("column")
                .short('c')
                .num_args(1)
                .default_value("1,2,3")
                .help("Columns of strain,species,genus (1-based)"),
        )
        .arg(
            Arg::new("separator")
                .long("separator")
                .short('s')
                .num_args(1)
                .default_value("\t")
                .help("Field separator"),
        )
        .arg(
            Arg::new("min")
                .long("min")
                .short('m')
                .num_args(1)
                .default_value("3")
                .value_parser(value_parser!(usize))
                .help("Minimal length for species abbreviation"),
        )
        .arg(
            Arg::new("tight")
                .long("tight")
                .action(ArgAction::SetTrue)
                .help("No underscore between genus and species"),
        )
        .arg(
            Arg::new("shortsub")
                .long("shortsub")
                .action(ArgAction::SetTrue)
                .help("Clean subspecies parts"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let column_str = args.get_one::<String>("column").unwrap();
    let cols: Vec<usize> = column_str
        .split(',')
        .map(|s| {
            s.parse()
                .map_err(|_| anyhow::anyhow!("Invalid column number: '{}'", s))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    if cols.len() != 3 {
        return Err(anyhow::anyhow!(
            "Column must be in format 's,p,g' (three numbers)"
        ));
    }
    for (i, col) in cols.iter().enumerate() {
        if *col == 0 {
            return Err(anyhow::anyhow!(
                "Column {} must be a positive integer (1-based)",
                i + 1
            ));
        }
    }

    nwr::libs::taxonomy::abbr::run(&nwr::libs::taxonomy::abbr::AbbrOptions {
        infile: args.get_one::<String>("infile").unwrap().clone(),
        outfile: args.get_one::<String>("outfile").unwrap().clone(),
        columns: (cols[0], cols[1], cols[2]),
        separator: args.get_one::<String>("separator").unwrap().clone(),
        min_len: *args.get_one("min").unwrap(),
        tight: args.get_flag("tight"),
        shortsub: args.get_flag("shortsub"),
    })
}

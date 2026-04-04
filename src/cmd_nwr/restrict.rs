use clap::*;
use intspan::IntSpan;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("restrict")
        .about("Restrict taxonomy terms to ancestral descendants")
        .after_help(include_str!("../../docs/help/restrict.md"))
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
            Arg::new("file")
                .long("file")
                .short('f')
                .num_args(1..)
                .action(ArgAction::Append)
                .default_value("stdin")
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("column")
                .long("column")
                .short('c')
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(usize))
                .help("The column where the IDs are located, starting from 1"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("exclude lines matching terms"),
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

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let column: usize = *args.get_one("column").unwrap();
    let is_exclude = args.get_flag("exclude");

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut id_set = IntSpan::new();
    for term in args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
    {
        let id = nwr::term_to_tax_id(&conn, term)?;
        let descendents: Vec<i32> = nwr::get_all_descendent(&conn, id)?
            .iter()
            .map(|n| *n as i32)
            .collect();
        id_set.add_vec(descendents.as_ref());
    }

    for infile in args
        .get_many::<String>("file")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
    {
        let reader = intspan::reader(infile);
        for line in reader.lines().map_while(Result::ok) {
            // Always output lines start with "#"
            if line.starts_with('#') {
                writer.write_fmt(format_args!("{}\n", line))?;
                continue;
            }

            // Check the given field
            let fields: Vec<&str> = line.split('\t').collect();
            let term = fields.get(column - 1).ok_or_else(|| {
                anyhow::anyhow!("Column {} not found in line: {}", column, line)
            })?;
            let id = nwr::term_to_tax_id(&conn, term)?;

            if is_exclude ^ id_set.contains(id as i32) {
                writer.write_fmt(format_args!("{}\n", line))?;
            }
        }
    }

    Ok(())
}

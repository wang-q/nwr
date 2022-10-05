use clap::*;
use intspan::IntSpan;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("restrict")
        .about("Restrict taxonomy terms to ancestral descendants")
        .after_help(
            r###"
* All terms, including ancestors and fields in input files,
  are in the form of a Taxonomy ID or scientific name.

* Input files should be TSV.
  * `tests/nwr/taxon.tsv` as an example.

* Lines start with `#` will always be outputted.

"###,
        )
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
                .help("Change working directory"),
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
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let column: usize = *args.get_one("column").unwrap();

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };

    let conn = nwr::connect_txdb(&nwrdir).unwrap();

    let mut id_set = IntSpan::new();
    for term in args.get_many::<String>("terms").unwrap() {
        let id = nwr::term_to_tax_id(&conn, term.to_string()).unwrap();
        let descendents: Vec<i32> = nwr::get_all_descendent(&conn, id)
            .unwrap()
            .iter()
            .map(|n| *n as i32)
            .collect();
        id_set.add_vec(descendents.as_ref());
    }

    for infile in args.get_many::<String>("file").unwrap() {
        let reader = intspan::reader(infile);
        for line in reader.lines().filter_map(|r| r.ok()) {
            // Always output lines start with "#"
            if line.starts_with("#") {
                writer.write_fmt(format_args!("{}\n", line))?;
                continue;
            }

            // Check the given field
            let fields: Vec<&str> = line.split('\t').collect();
            let term = fields.get(column - 1).unwrap();
            let id = nwr::term_to_tax_id(&conn, term.to_string()).unwrap();

            if id_set.contains(id as i32) {
                writer.write_fmt(format_args!("{}\n", line))?;
            }
        }
    }

    Ok(())
}

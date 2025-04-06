use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("subset")
        .about("Extract a submatrix from a PHYLIP matrix using a list of names")
        .after_help(
            r###"
Input formats:
    * Matrix: PHYLIP distance matrix (full or lower-triangular)
    * List: One name per line
    * Empty lines and lines starting with '#' in the list file are ignored

Examples:
    1. Create a full submatrix:
       nwr mat subset input.phy list.txt -o output.phy

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input PHYLIP matrix file"),
        )
        .arg(
            Arg::new("list")
                .required(true)
                .index(2)
                .help("File containing sequence names to extract"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Args
    //----------------------------
    let infile = args.get_one::<String>("infile").unwrap();
    let list_file = args.get_one::<String>("list").unwrap();
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let wanted_names = intspan::read_first_column(list_file);

    //----------------------------
    // Load and process matrix
    //----------------------------
    let matrix = nwr::NamedMatrix::from_relaxed_phylip(infile);
    let all_names = matrix.get_names();
    let mut indices = Vec::new();

    // Find indices of wanted names
    for name in &wanted_names {
        if let Some(idx) = all_names.iter().position(|n| *n == name) {
            indices.push(idx);
        } else {
            eprintln!("Warning: Name not found in matrix: {}", name);
        }
    }

    // Write sequence count
    writer.write_fmt(format_args!("{}\n", indices.len()))?;

    // Output submatrix
    for &i in &indices {
        writer.write_fmt(format_args!("{}", all_names[i]))?;
        for &j in &indices {
            writer.write_fmt(format_args!("\t{}", matrix.get(i, j)))?;
        }

        writer.write_fmt(format_args!("\n"))?;
    }

    Ok(())
}

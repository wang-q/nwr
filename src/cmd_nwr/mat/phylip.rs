use clap::*;
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("phylip")
        .about("Convert pairwise distances to a phylip distance matrix")
        .after_help(
            r###"
Input format:
    * Tab-separated values (TSV)
    * Three columns: name1, name2, distance

Examples:
    1. Convert pairwise distances to PHYLIP matrix:
       nwr mat phylip input.tsv -o output.phy
"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input file containing pairwise distances"),
        )
        .arg(
            Arg::new("same")
                .long("same")
                .num_args(1)
                .default_value("0.0")
                .value_parser(value_parser!(f32))
                .help("Default score of identical element pairs"),
        )
        .arg(
            Arg::new("missing")
                .long("missing")
                .num_args(1)
                .default_value("1.0")
                .value_parser(value_parser!(f32))
                .help("Default score of missing pairs"),
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

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Args
    //----------------------------
    let infile = args.get_one::<String>("infile").unwrap();
    let opt_same = *args.get_one::<f32>("same").unwrap();
    let opt_missing = *args.get_one::<f32>("missing").unwrap();
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    //----------------------------
    // Ops
    //----------------------------
    // Load matrix from pairwise distances
    let matrix = nwr::NamedMatrix::from_pair_scores(infile, opt_same, opt_missing);
    let names = matrix.get_names();
    let size = matrix.size();

    // Write sequence count
    writer.write_fmt(format_args!("{:>4}\n", size))?;

    // Output full matrix
    for i in 0..size {
        writer.write_fmt(format_args!("{}", names[i]))?;
        for j in 0..size {
            writer.write_fmt(format_args!("\t{}", matrix.get(i, j)))?;
        }
        writer.write_fmt(format_args!("\n"))?;
    }

    Ok(())
}

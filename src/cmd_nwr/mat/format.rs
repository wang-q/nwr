use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("format")
        .about("Convert between different PHYLIP matrix formats")
        .after_help(
            r###"
Convert a PHYLIP matrix between different formats while preserving all distance values.

Input format:
    * PHYLIP distance matrix (full or lower-triangular)
    * Optional first line: number of sequences
    * Each line: sequence name followed by distances

Output formats:
    * full
        - Full square matrix
        - Tab-separated values
        - Original sequence names preserved
    * lower
        - Lower triangular matrix
        - Tab-separated values
        - Original sequence names preserved
    * strict
        - Standard PHYLIP format
        - Names truncated to 10 characters
        - Names left-aligned with space padding
        - Distances in fixed-width format (6 decimal places)
        - Space-separated values

Examples:
    1. Create a full matrix:
       nwr mat format input.phy -o output.phy

    2. Create a lower-triangular matrix:
       nwr mat format input.phy --mode lower -o output.phy

    3. Create a strict PHYLIP matrix:
       nwr mat format input.phy --mode strict -o output.phy
"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input PHYLIP matrix file"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("full"),
                    builder::PossibleValue::new("lower"),
                    builder::PossibleValue::new("strict"),
                ])
                .default_value("full")
                .help("Output format"),
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
    let opt_mode = args.get_one::<String>("mode").unwrap();
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    //----------------------------
    // Ops
    //----------------------------
    let matrix = intspan::NamedMatrix::from_relaxed_phylip(infile);
    let names = matrix.get_names();
    let size = matrix.size();

    // Write sequence count
    writer.write_fmt(format_args!("{:>4}\n", size))?;

    for i in 0..size {
        match opt_mode.as_str() {
            "full" => {
                writer.write_fmt(format_args!("{}", names[i]))?;
                for j in 0..size {
                    writer.write_fmt(format_args!("\t{}", matrix.get(i, j)))?;
                }
            }
            "lower" => {
                writer.write_fmt(format_args!("{}", names[i]))?;
                for j in 0..i {
                    writer.write_fmt(format_args!("\t{}", matrix.get(i, j)))?;
                }
            }
            "strict" => {
                writer.write_fmt(format_args!(
                    "{:<10}",
                    names[i].chars().take(10).collect::<String>()
                ))?;
                for j in 0..size {
                    writer.write_fmt(format_args!(" {:.6}", matrix.get(i, j)))?;
                }
            }
            _ => unreachable!(),
        }
        writer.write_fmt(format_args!("\n"))?;
    }

    Ok(())
}

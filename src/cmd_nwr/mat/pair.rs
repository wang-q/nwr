use clap::*;
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> clap::Command {
    clap::Command::new("pair")
        .about("Convert a PHYLIP distance matrix to pairwise distances")
        .after_help(
            r###"
This command converts a (relaxed lower-triangular) PHYLIP-format distance matrix
to pairwise distances.

Input format:
* PHYLIP distance matrix (full or lower-triangular)
* First line can be sequence count (optional)
* Each line: sequence name followed by distances

Output format:
* Tab-separated values (TSV)
* Three columns: name1, name2, distance
* Symmetric output (both directions included)

Examples:
1. Convert a PHYLIP matrix:
   nwr mat pair input.mat -o output.tsv

2. Output to screen:
   nwr mat pair input.mat

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input file containing a PHYLIP distance matrix"),
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
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    //----------------------------
    // Ops
    //----------------------------
    // Load matrix from PHYLIP format
    let matrix = nwr::NamedMatrix::from_relaxed_phylip(infile);
    let names = matrix.get_names();

    // Output pairwise distances (lower triangle only)
    for i in 0..matrix.size() {
        for j in 0..=i {
            let distance = matrix.get(i, j);
            writer
                .write_fmt(format_args!("{}\t{}\t{}\n", names[j], names[i], distance))?;
        }
    }

    Ok(())
}

// // Process a single line of the PHYLIP matrix and output pairwise distances
// fn process_phylip_line(
//     line: &str,
//     names: &mut Vec<String>,
//     writer: &mut Box<dyn Write>,
// ) -> anyhow::Result<()> {
//     let parts: Vec<&str> = line.trim().split_whitespace().collect();
//     if !parts.is_empty() {
//         let name = parts[0].to_string();
//         names.push(name.clone());

//         // Read lower-triangle distances
//         let distances: Vec<f32> = parts[1..=names.len()]
//             .iter()
//             .map(|&s| s.parse().unwrap())
//             .collect();

//         for (i, &distance) in distances.iter().enumerate() {
//             writer.write_fmt(format_args!("{}\t{}\t{}\n", names[i], name, distance))?;
//         }
//     }

//     Ok(())
// }

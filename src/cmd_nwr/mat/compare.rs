use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("compare")
        .about("Compare two distance matrices")
        .after_help(
            r###"
Compare two PHYLIP distance matrices and calculate similarity metrics.

Methods:
    * all:       Calculate all metrics below
    * pearson:   Pearson correlation coefficient (-1 to 1)
    * spearman:  Spearman rank correlation (-1 to 1)
    * mae:       Mean absolute error
    * cosine:    Cosine similarity (-1 to 1)
    * jaccard:   Weighted Jaccard similarity (0 to 1)
    * euclid:    Euclidean distance

Examples:
    # Compare using Pearson correlation
    nwr mat compare matrix1.phy matrix2.phy --method pearson

    # Compare using multiple methods
    nwr mat compare matrix1.phy matrix2.phy --method pearson,cosine,jaccard
"###,
        )
        .arg(
            Arg::new("matrix1")
                .required(true)
                .index(1)
                .help("First PHYLIP matrix file"),
        )
        .arg(
            Arg::new("matrix2")
                .required(true)
                .index(2)
                .help("Second PHYLIP matrix file"),
        )
        .arg(
            Arg::new("method")
                .long("method")
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("all"),
                    builder::PossibleValue::new("pearson"),
                    builder::PossibleValue::new("spearman"),
                    builder::PossibleValue::new("mae"),
                    builder::PossibleValue::new("cosine"),
                    builder::PossibleValue::new("jaccard"),
                    builder::PossibleValue::new("euclid"),
                ])
                .default_value("pearson")
                .help("Comparison method(s), comma-separated"),
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
    let matrix1_file = args.get_one::<String>("matrix1").unwrap();
    let matrix2_file = args.get_one::<String>("matrix2").unwrap();
    let methods = if args.get_one::<String>("method").unwrap() == "all" {
        "pearson,spearman,mae,cosine,jaccard,euclid"
    } else {
        args.get_one::<String>("method").unwrap()
    };
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    // Load matrices
    let matrix1 = nwr::NamedMatrix::from_relaxed_phylip(matrix1_file);
    let matrix2 = nwr::NamedMatrix::from_relaxed_phylip(matrix2_file);

    // Get common sequence names
    let names1 = matrix1.get_names();
    let names2 = matrix2.get_names();
    let common_names: Vec<_> =
        names1.iter().filter(|name| names2.contains(name)).collect();

    // Report sequence counts
    eprintln!(
        "Sequences in matrices: {} and {}",
        names1.len(),
        names2.len()
    );
    eprintln!("Common sequences: {}", common_names.len());

    if common_names.is_empty() {
        return Err(anyhow::anyhow!(
            "No common sequence names found between matrices"
        ));
    }

    // Extract values for comparison
    let mut values1 =
        Vec::with_capacity(common_names.len() * (common_names.len() - 1) / 2);
    let mut values2 =
        Vec::with_capacity(common_names.len() * (common_names.len() - 1) / 2);

    for i in 0..common_names.len() {
        for j in 0..i {
            if let (Some(v1), Some(v2)) = (
                matrix1.get_by_name(&common_names[i], &common_names[j]),
                matrix2.get_by_name(&common_names[i], &common_names[j]),
            ) {
                values1.push(v1);
                values2.push(v2);
            }
        }
    }

    // Write header
    writer.write_all(b"Method\tScore\n")?;

    // Calculate and output metrics
    for method in methods.split(',') {
        let result = match method {
            "pearson" => nwr::pearson_correlation(&values1, &values2),
            "spearman" => spearman_correlation(&values1, &values2),
            "mae" => mean_absolute_error(&values1, &values2),
            "cosine" => nwr::cosine_similarity(&values1, &values2),
            "jaccard" => nwr::weighted_jaccard_similarity(&values1, &values2),
            "euclid" => nwr::euclidean_distance(&values1, &values2),
            _ => unreachable!(),
        };
        writer.write_fmt(format_args!("{}\t{:.6}\n", method, result))?;
    }

    Ok(())
}

// Calculate Spearman rank correlation coefficient
fn spearman_correlation(x: &[f32], y: &[f32]) -> f32 {
    let mut x_ranked: Vec<_> = x.iter().enumerate().collect();
    let mut y_ranked: Vec<_> = y.iter().enumerate().collect();

    x_ranked.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
    y_ranked.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    let mut x_ranks = vec![0.0; x.len()];
    let mut y_ranks = vec![0.0; y.len()];

    for (rank, &(i, _)) in x_ranked.iter().enumerate() {
        x_ranks[i] = rank as f32;
    }
    for (rank, &(i, _)) in y_ranked.iter().enumerate() {
        y_ranks[i] = rank as f32;
    }

    nwr::pearson_correlation(&x_ranks, &y_ranks)
}

// Calculate mean absolute error
fn mean_absolute_error(x: &[f32], y: &[f32]) -> f32 {
    x.iter()
        .zip(y.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / x.len() as f32
}

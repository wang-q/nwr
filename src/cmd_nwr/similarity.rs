use clap::*;
use rayon::prelude::*;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("similarity")
        .about("Similarity of vectors")
        .after_help(
            r###"
modes:
    * euclidean distance
        * --mode euclid
    * euclidean distance to similarity
        * --mode euclid --sim
    * binary euclidean distance
        * --mode euclid --bin
    * binary euclidean distance to dissimilarity
        * --mode euclid --bin --sim --dis

    * cosine similarity, -1 -- 1
        * --mode cosine
    * cosine distance, 0 -- 2
        * --mode cosine --dis
    * binary cosine similarity
        * --mode cosine --bin
    * binary cosine similarity
        * --mode cosine --bin --dis

    * jaccard index
        * --mode jaccard --bin
    * weighted jaccard similarity
        * --mode jaccard

"###,
        )
        .arg(
            Arg::new("infiles")
                .required(true)
                .num_args(1..=2)
                .index(1)
                .required(true)
                .help("Input filenames. [stdin] for standard input"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("euclid"),
                    builder::PossibleValue::new("cosine"),
                    builder::PossibleValue::new("jaccard"),
                ])
                .default_value("euclid")
                .help("Where we can find taxonomy terms"),
        )
        .arg(
            Arg::new("bin")
                .long("bin")
                .action(ArgAction::SetTrue)
                .help("Treat values in list as 0,1"),
        )
        .arg(
            Arg::new("sim")
                .long("sim")
                .action(ArgAction::SetTrue)
                .help("Convert distance to similarity"),
        )
        .arg(
            Arg::new("dis")
                .long("dis")
                .action(ArgAction::SetTrue)
                .help("Convert to dissimilarity"),
        )
        .arg(
            Arg::new("parallel")
                .long("parallel")
                .num_args(1)
                .default_value("8")
                .value_parser(value_parser!(usize))
                .help("Number of threads"),
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
    //----------------------------
    // Args
    //----------------------------
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let mode = args.get_one::<String>("mode").unwrap();

    let is_bin = args.get_flag("bin");
    let is_sim = args.get_flag("sim");
    let is_dis = args.get_flag("dis");

    let parallel = *args.get_one::<usize>("parallel").unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(parallel)
        .build_global()
        .unwrap();

    let infiles = args
        .get_many::<String>("infiles")
        .unwrap()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    //----------------------------
    // Ops
    //----------------------------
    if infiles.len() == 1 {
        let entries = load_file(infiles.get(0).unwrap(), is_bin);

        for i in 0..entries.len() {
            for j in 0..entries.len() {
                let e1 = entries.get(i).unwrap();
                let e2 = entries.get(j).unwrap();
                let score = calc(e1.list(), e2.list(), mode, is_sim, is_dis);

                writer.write_fmt(format_args!(
                    "{}\t{}\t{:.4}\n",
                    e1.name(),
                    e2.name(),
                    score
                ))?;
            }
        }
    } else if infiles.len() == 2 {
        let entries = load_file(infiles.get(0).unwrap(), is_bin);
        let others = load_file(infiles.get(1).unwrap(), is_bin);

        for e1 in &entries {
            for e2 in &others {
                let score = calc(e1.list(), e2.list(), mode, is_sim, is_dis);

                writer.write_fmt(format_args!(
                    "{}\t{}\t{:.4}\n",
                    e1.name(),
                    e2.name(),
                    score
                ))?;
            }
        }
    }

    Ok(())
}

fn load_file(infile: &str, is_bin: bool) -> Vec<nwr::AsmEntry> {
    let mut entries = vec![];
    let reader = intspan::reader(infile);
    'LINE: for line in reader.lines().map_while(Result::ok) {
        let mut entry = nwr::AsmEntry::parse(&line);
        if entry.name().is_empty() {
            continue 'LINE;
        }
        if is_bin {
            let bin_list = entry
                .list()
                .iter()
                .map(|e| if *e > 0.0 { 1.0 } else { 0.0 })
                .collect::<Vec<f64>>();
            entry = nwr::AsmEntry::from(entry.name(), &bin_list);
        }
        entries.push(entry);
    }
    entries
}

fn calc(l1: &[f64], l2: &[f64], mode: &str, is_sim: bool, is_dis: bool) -> f64 {
    let mut score = match mode {
        "euclid" => euclidean_distance(l1, l2),
        "cosine" => cosine_similarity(l1, l2),
        "jaccard" => weighted_jaccard_similarity(l1, l2),
        _ => unreachable!(),
    };

    if is_sim {
        score = d2s(score);
    }
    if is_dis {
        score = dis(score);
    }

    score
}

// https://www.maartengrootendorst.com/blog/distances/
// https://crates.io/crates/semanticsimilarity_rs
fn euclidean_distance(v1: &[f64], v2: &[f64]) -> f64 {
    v1.par_iter()
        .zip(v2.par_iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn dot_product(v1: &[f64], v2: &[f64]) -> f64 {
    v1.par_iter().zip(v2.par_iter()).map(|(a, b)| a * b).sum()
}

fn norm(v1: &[f64]) -> f64 {
    v1.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt()
}

fn cosine_similarity(v1: &[f64], v2: &[f64]) -> f64 {
    let dot_product = dot_product(v1, v2);
    let denominator = norm(v1) * norm(v2);

    if denominator == 0.0 {
        0.0
    } else {
        dot_product / denominator
    }
}

fn weighted_jaccard_similarity(v1: &[f64], v2: &[f64]) -> f64 {
    let numerator = v1
        .par_iter()
        .zip(v2.par_iter())
        .map(|(a, b)| f64::min(*a, *b))
        .sum::<f64>();
    let denominator = v1
        .par_iter()
        .zip(v2.par_iter())
        .map(|(a, b)| f64::max(*a, *b))
        .sum::<f64>();

    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

// Schölkopf, B. (2000). The kernel trick for distances. In Neural Information Processing Systems, pages 301-307.
// https://stats.stackexchange.com/questions/146309/turn-a-distance-measure-into-a-kernel-function
// https://stats.stackexchange.com/questions/158279/how-i-can-convert-distance-euclidean-to-similarity-score
fn d2s(dist: f64) -> f64 {
    1.0 / dist.abs().exp()
}

fn dis(dist: f64) -> f64 {
    1.0 - dist
}

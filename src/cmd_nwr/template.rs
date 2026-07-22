use super::args;
use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("template")
        .about("Creates dirs, data and scripts for a phylogenomic research")
        .after_help(include_str!("../../docs/help/template.md"))
        // Global
        .arg(args::infiles_arg(".assembly.tsv files"))
        .arg(args::outdir_arg())
        .arg(
            Arg::new("include")
                .long("include")
                .num_args(1..)
                .action(ArgAction::Append)
                .help(
                    "Only the assemblies *in* these lists in the MinHash, Count and Protein steps",
                ),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Only the assemblies *not in* these lists"),
        )
        .arg(
            Arg::new("parallel")
                .long("parallel")
                .num_args(1)
                .default_value("8")
                .value_parser(value_parser!(usize))
                .help("Number of threads"),
        )
        // ASSEMBLY
        .arg(
            Arg::new("ass")
                .long("ass")
                .action(ArgAction::SetTrue)
                .help("Prepare ASSEMBLY/ materials"),
        )
        // BioSample
        .arg(
            Arg::new("bs")
                .long("bs")
                .action(ArgAction::SetTrue)
                .help("Prepare BioSample/ materials"),
        )
        // MinHash
        .arg(
            Arg::new("mh")
                .long("mh")
                .action(ArgAction::SetTrue)
                .help("Prepare MinHash/ materials"),
        )
        .arg(
            Arg::new("sketch")
                .long("sketch")
                .num_args(1)
                .default_value("10000")
                .value_parser(value_parser!(usize))
                .help("Sketch size passed to `mash sketch`"),
        )
        .arg(
            Arg::new("ani-ab")
                .long("ani-ab")
                .num_args(1)
                .default_value("0.05")
                .value_parser(value_parser!(f64))
                .help("The ANI value for abnormal strains"),
        )
        .arg(
            Arg::new("ani-nr")
                .long("ani-nr")
                .num_args(1)
                .default_value("0.005")
                .value_parser(value_parser!(f64))
                .help("The ANI value for non-redundant strains"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .num_args(1)
                .default_value("0.5")
                .value_parser(value_parser!(f64))
                .help("Height value passed to `cutree()`"),
        )
        // Count
        .arg(
            Arg::new("count")
                .long("count")
                .action(ArgAction::SetTrue)
                .help("Prepare Count/ materials"),
        )
        .arg(args::rank_arg())
        .arg(
            Arg::new("lineage")
                .long("lineage")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("To list which rank(s) in the lineage"),
        )
        // Protein
        .arg(
            Arg::new("pro")
                .long("pro")
                .action(ArgAction::SetTrue)
                .help("Prepare Protein/ materials"),
        )
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outdir = args.get_one::<String>("outdir").unwrap().clone();

    let ins: Vec<String> = args
        .get_many::<String>("include")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let not_ins: Vec<String> = args
        .get_many::<String>("exclude")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let infiles: Vec<String> = args
        .get_many::<String>("infiles")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    let ranks: Vec<String> = args
        .get_many::<String>("rank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let lineages: Vec<String> = args
        .get_many::<String>("lineage")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    nwr::libs::template::run(&nwr::libs::template::TemplateOptions {
        outdir,
        infiles,
        ins,
        not_ins,
        parallel: *args.get_one::<usize>("parallel").unwrap(),
        sketch: *args.get_one::<usize>("sketch").unwrap(),
        ani_ab: *args.get_one::<f64>("ani-ab").unwrap(),
        ani_nr: *args.get_one::<f64>("ani-nr").unwrap(),
        height: *args.get_one::<f64>("height").unwrap(),
        ranks,
        lineages,
        do_ass: args.get_flag("ass"),
        do_bs: args.get_flag("bs"),
        do_mh: args.get_flag("mh"),
        do_count: args.get_flag("count"),
        do_pro: args.get_flag("pro"),
    })
}

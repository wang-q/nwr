use clap::*;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("template")
        .about("Creates dirs, data and scripts for a phylogenomic research")
        .after_help(include_str!("../../docs/help/template.md"))
        // Global
        .arg(
            Arg::new("infiles")
                .help(".assembly.tsv files")
                .num_args(1..)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("outdir")
                .long("outdir")
                .short('o')
                .num_args(1)
                .default_value(".")
                .help("Output directory (default: current directory)"),
        )
        .arg(
            Arg::new("in")
                .long("in")
                .num_args(1..)
                .action(ArgAction::Append)
                .help(
                    "Only the assemblies *in* these lists in the MinHash, Count and Protein steps",
                ),
        )
        .arg(
            Arg::new("not-in")
                .long("not-in")
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
        .arg(
            Arg::new("rank")
                .long("rank")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("To list which rank(s) - genus, family, order, and class"),
        )
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

    let mut ins = vec![];
    if args.contains_id("in") {
        for i in args.get_many::<String>("in").unwrap() {
            ins.push(i.clone());
        }
    }

    let mut not_ins = vec![];
    if args.contains_id("not-in") {
        for i in args.get_many::<String>("not-in").unwrap() {
            not_ins.push(i.clone());
        }
    }

    let mut infiles = vec![];
    if args.contains_id("infiles") {
        for i in args.get_many::<String>("infiles").unwrap() {
            infiles.push(i.clone());
        }
    }

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.clone());
        }
    }

    let mut lineages = vec![];
    if args.contains_id("lineage") {
        for rank in args.get_many::<String>("lineage").unwrap() {
            lineages.push(rank.clone());
        }
    }

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

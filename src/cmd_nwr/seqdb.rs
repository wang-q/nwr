use clap::*;
use simplelog::*;
use std::path::PathBuf;

/// Resolve an optional file argument.
///
/// If the argument was not provided, returns `None`.
/// If it was provided without a value, returns the default path under `dir`.
/// If it was provided with a value, returns that path.
fn opt_path(
    args: &ArgMatches,
    name: &str,
    dir: &std::path::Path,
    default: &str,
) -> Option<PathBuf> {
    if args.contains_id(name) {
        Some(match args.get_one::<String>(name) {
            Some(path) => PathBuf::from(path),
            None => dir.join(default),
        })
    } else {
        None
    }
}

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("seqdb")
        .about("Init the seq database")
        .after_help(include_str!("../../docs/help/seqdb.md"))
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .default_value(".")
                .help("Specify the working directory"),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .action(ArgAction::SetTrue)
                .help("Initialize (delete) the database"),
        )
        .arg(
            Arg::new("strain")
                .long("strain")
                .num_args(0..=1)
                .help("Load strains.tsv file"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .num_args(0..=1)
                .help("Load sizes.tsv file"),
        )
        .arg(
            Arg::new("clust")
                .long("clust")
                .num_args(0..=1)
                .help("Load rep_cluster.tsv file"),
        )
        .arg(
            Arg::new("anno")
                .long("anno")
                .num_args(0..=1)
                .help("Load anno.tsv file"),
        )
        .arg(
            Arg::new("asmseq")
                .long("asmseq")
                .num_args(0..=1)
                .help("Load asmseq.tsv file"),
        )
        .arg(
            Arg::new("rep")
                .long("rep")
                .num_args(1)
                .help("Load features into rep table (format: field=file)"),
        )
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Debug, Config::default())?;

    let dir = std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf();
    let opt_strain = opt_path(args, "strain", &dir, "strains.tsv");
    let opt_size = opt_path(args, "size", &dir, "sizes.tsv");
    let opt_clust = opt_path(args, "clust", &dir, "rep_cluster.tsv");
    let opt_anno = opt_path(args, "anno", &dir, "anno.tsv");
    let opt_asmseq = opt_path(args, "asmseq", &dir, "asmseq.tsv");
    let opt_rep = if args.contains_id("rep") {
        let rep = args.get_one::<String>("rep").unwrap();
        let pos = rep.find('=').ok_or_else(|| {
            anyhow::anyhow!("invalid KEY=value: no `=` found in `{rep}`")
        })?;
        Some((rep[..pos].to_string(), rep[pos + 1..].to_string()))
    } else {
        None
    };

    nwr::libs::db::seqdb::run(&nwr::libs::db::seqdb::SeqdbOptions {
        dir,
        is_init: args.get_flag("init"),
        opt_strain,
        opt_size,
        opt_clust,
        opt_anno,
        opt_asmseq,
        opt_rep: opt_rep.map(|(field, path)| (field, PathBuf::from(path))),
    })
}

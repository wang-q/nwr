use clap::*;
use log::info;
use simplelog::*;
use std::fs::File;
use std::path::PathBuf;

use nwr::libs::db::seqdb::{
    insert_anno, insert_asmseq, insert_clust, insert_rep, insert_size, insert_strain,
    DDL_SEQ, VALID_REP_FIELDS,
};

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
        .about("Initializes the seq database")
        .after_help(include_str!("../../docs/help/seqdb.md"))
        .arg(
            Arg::new("workdir")
                .long("workdir")
                .short('w')
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
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let dir =
        std::path::Path::new(args.get_one::<String>("workdir").unwrap()).to_path_buf();
    let is_init = args.get_flag("init");
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
        if pos == 0 || pos + 1 >= rep.len() {
            anyhow::bail!("invalid KEY=value: empty field or path in `{rep}`");
        }
        let field = &rep[..pos];
        if !VALID_REP_FIELDS.contains(&field) {
            anyhow::bail!(
                "Invalid rep field '{}'. Valid fields are: {:?}",
                field,
                VALID_REP_FIELDS
            );
        }
        Some((rep[..pos].to_string(), PathBuf::from(&rep[pos + 1..])))
    } else {
        None
    };

    let db = dir.join("seq.sqlite");
    if is_init && db.exists() {
        std::fs::remove_file(&db)?;
    }

    info!("==> Opening database `{}`", db.display());
    let conn = rusqlite::Connection::open(db)?;
    conn.execute_batch(
        "
        -- To improve performance
        -- disables the rollback journal
        PRAGMA journal_mode = OFF;
        -- SQLite will not wait for data to be written to disk
        PRAGMA synchronous = 0;
        -- reducing disk I/O
        PRAGMA cache_size = 1000000;
        -- reducing lock contention, as no others would use the db
        PRAGMA locking_mode = EXCLUSIVE;
        -- stores temporary tables and indices in memory
        PRAGMA temp_store = MEMORY;
        ",
    )?;

    if is_init {
        info!("==> Create tables");
        conn.execute_batch(DDL_SEQ)?;
    }

    if let Some(opt_strain) = &opt_strain {
        info!("==> Loading `{}` to `rank` and `asm`", opt_strain.display());
        let dmp: File = File::open(opt_strain)?;
        insert_strain(&dmp, opt_strain, &conn)?;
    }

    if let Some(opt_size) = &opt_size {
        info!("==> Loading `{}` to `seq`", opt_size.display());
        let dmp = File::open(opt_size)?;
        insert_size(&dmp, opt_size, &conn)?;
    }

    if let Some(opt_clust) = &opt_clust {
        info!(
            "==> Loading `{}` to `rep` and `rep_seq`",
            opt_clust.display()
        );
        let dmp = File::open(opt_clust)?;
        insert_clust(&dmp, opt_clust, &conn)?;
    }

    if let Some(opt_anno) = &opt_anno {
        info!("==> Loading `{}` to `seq`", opt_anno.display());
        let dmp = File::open(opt_anno)?;
        insert_anno(&dmp, opt_anno, &conn)?;
    }

    if let Some(opt_asmseq) = &opt_asmseq {
        info!("==> Loading `{}` to `asm_seq`", opt_asmseq.display());
        let dmp = File::open(opt_asmseq)?;
        insert_asmseq(&dmp, opt_asmseq, &conn)?;
    }

    if let Some((rep_field, rep_path)) = &opt_rep {
        info!(
            "==> Loading `{}` to `rep.{}`",
            rep_path.display(),
            rep_field
        );
        let dmp = File::open(rep_path)?;
        insert_rep(&dmp, rep_field, rep_path, &conn)?;
    }

    Ok(())
}

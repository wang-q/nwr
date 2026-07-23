use clap::{Arg, ArgAction, ArgMatches, Command};
use log::info;

use std::fs::File;
use std::path::PathBuf;

use nwr::libs::seqdb::{
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
        Some(
            args.get_one::<String>(name)
                .map_or_else(|| dir.join(default), PathBuf::from),
        )
    } else {
        None
    }
}

/// Create clap subcommand arguments.
#[must_use]
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

/// Load a TSV file into the database when `opt` is present.
fn load_file<F>(
    opt: Option<&PathBuf>,
    description: &str,
    insert: F,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()>
where
    F: FnOnce(&File, &std::path::Path, &rusqlite::Connection) -> anyhow::Result<()>,
{
    if let Some(path) = opt {
        info!("==> Loading `{}` {description}", path.display());
        let file = File::open(path)?;
        insert(&file, path, conn)?;
    }
    Ok(())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    nwr::libs::io::init_logger();

    let dir = std::path::Path::new(
        args.get_one::<String>("workdir")
            .ok_or_else(|| anyhow::anyhow!("Missing 'workdir' argument"))?,
    )
    .to_path_buf();
    let is_init = args.get_flag("init");
    let opt_strain = opt_path(args, "strain", &dir, "strains.tsv");
    let opt_size = opt_path(args, "size", &dir, "sizes.tsv");
    let opt_clust = opt_path(args, "clust", &dir, "rep_cluster.tsv");
    let opt_anno = opt_path(args, "anno", &dir, "anno.tsv");
    let opt_asmseq = opt_path(args, "asmseq", &dir, "asmseq.tsv");
    let opt_rep = if args.contains_id("rep") {
        let rep = args
            .get_one::<String>("rep")
            .ok_or_else(|| anyhow::anyhow!("Missing 'rep' argument"))?;
        let pos = rep.find('=').ok_or_else(|| {
            anyhow::anyhow!("invalid KEY=value: no `=` found in `{rep}`")
        })?;
        if pos == 0 || pos + 1 >= rep.len() {
            anyhow::bail!("invalid KEY=value: empty field or path in `{rep}`");
        }
        let field = &rep[..pos];
        if !VALID_REP_FIELDS.contains(&field) {
            anyhow::bail!(
                "Invalid rep field '{field}'. Valid fields are: {VALID_REP_FIELDS:?}"
            );
        }
        Some((field.to_string(), PathBuf::from(&rep[pos + 1..])))
    } else {
        None
    };

    let db = dir.join("seq.sqlite");
    if is_init && db.exists() {
        std::fs::remove_file(&db)?;
    }

    info!("==> Opening database `{}`", db.display());
    let conn = rusqlite::Connection::open(db)?;
    nwr::libs::db::apply_import_pragmas(&conn)?;

    if is_init {
        info!("==> Create tables");
        conn.execute_batch(DDL_SEQ)?;
    }

    // Guard against accidentally operating on an empty database without
    // --init, which would otherwise produce confusing "no such table" errors.
    let table_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='seq'",
        [],
        |row| row.get(0),
    )?;
    if table_count == 0 && !is_init {
        anyhow::bail!(
            "seq.sqlite is empty or missing required tables; use --init to create them"
        );
    }

    load_file(
        opt_strain.as_ref(),
        "to `rank` and `asm`",
        insert_strain,
        &conn,
    )?;
    load_file(opt_size.as_ref(), "to `seq`", insert_size, &conn)?;
    load_file(
        opt_clust.as_ref(),
        "to `rep` and `rep_seq`",
        insert_clust,
        &conn,
    )?;
    load_file(opt_anno.as_ref(), "to `seq`", insert_anno, &conn)?;
    load_file(opt_asmseq.as_ref(), "to `asm_seq`", insert_asmseq, &conn)?;

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

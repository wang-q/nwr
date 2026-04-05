use clap::*;
use log::info;
use simplelog::*;
use std::io::Write;
use std::path::PathBuf;

/// Valid field names for the rep table
const VALID_REP_FIELDS: &[&str] = &["f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8"];

/// Validate that a field name is allowed for the rep table
fn validate_rep_field(field: &str) -> anyhow::Result<&str> {
    if VALID_REP_FIELDS.contains(&field) {
        Ok(field)
    } else {
        Err(anyhow::anyhow!(
            "Invalid field name '{}'. Valid fields are: {:?}",
            field,
            VALID_REP_FIELDS
        ))
    }
}

// Create clap subcommand arguments
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

// https://stackoverflow.com/questions/58684279/can-an-index-on-a-text-column-speed-up-prefix-based-like-queries
static DDL_SEQ: &str = r###"
CREATE TABLE rank (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE
);
-- assembly
CREATE TABLE asm (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    rank_id INTEGER NOT NULL,
    FOREIGN KEY (rank_id) REFERENCES rank(id)
);
-- sequence
CREATE TABLE seq (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    size INTEGER,
    anno TEXT
);
-- representative
CREATE TABLE rep (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    f1 TEXT,
    f2 TEXT,
    f3 TEXT,
    f4 TEXT,
    f5 TEXT,
    f6 TEXT,
    f7 TEXT,
    f8 TEXT
);
-- Junction table to associate rep with seq
CREATE TABLE rep_seq (
    rep_id INTEGER NOT NULL,
    seq_id INTEGER NOT NULL,
    PRIMARY KEY (rep_id, seq_id),
    FOREIGN KEY (rep_id) REFERENCES rep(id),
    FOREIGN KEY (seq_id) REFERENCES seq(id)
);
-- Junction table to associate asm with seq
CREATE TABLE asm_seq (
    asm_id INTEGER NOT NULL,
    seq_id INTEGER NOT NULL,
    PRIMARY KEY (asm_id, seq_id),
    FOREIGN KEY (asm_id) REFERENCES asm(id),
    FOREIGN KEY (seq_id) REFERENCES seq(id)
);
-- Regular indices
CREATE INDEX rep_idx_f1 ON rep(f1);
CREATE INDEX rep_idx_f2 ON rep(f2);
CREATE INDEX rep_idx_f3 ON rep(f3);
CREATE INDEX rep_idx_f4 ON rep(f4);
CREATE INDEX rep_idx_f5 ON rep(f5);
CREATE INDEX rep_idx_f6 ON rep(f6);
CREATE INDEX rep_idx_f7 ON rep(f7);
CREATE INDEX rep_idx_f8 ON rep(f8);
-- Case-insensitive indices for `like`
CREATE INDEX seq_idx_anno ON seq(anno COLLATE NOCASE);

"###;

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Args
    //----------------------------
    let dir = std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf();

    let is_init = args.get_flag("init");
    let opt_strain = if args.contains_id("strain") {
        match args.get_one::<String>("strain") {
            Some(path) => PathBuf::from(path),
            None => dir.join("strains.tsv"),
        }
    } else {
        PathBuf::new()
    };
    let opt_size = if args.contains_id("size") {
        match args.get_one::<String>("size") {
            Some(path) => PathBuf::from(path),
            None => dir.join("sizes.tsv"),
        }
    } else {
        PathBuf::new()
    };
    let opt_clust = if args.contains_id("clust") {
        match args.get_one::<String>("clust") {
            Some(path) => PathBuf::from(path),
            None => dir.join("rep_cluster.tsv"),
        }
    } else {
        PathBuf::new()
    };
    let opt_anno = if args.contains_id("anno") {
        match args.get_one::<String>("anno") {
            Some(path) => PathBuf::from(path),
            None => dir.join("anno.tsv"),
        }
    } else {
        PathBuf::new()
    };
    let opt_asmseq = if args.contains_id("asmseq") {
        match args.get_one::<String>("asmseq") {
            Some(path) => PathBuf::from(path),
            None => dir.join("asmseq.tsv"),
        }
    } else {
        PathBuf::new()
    };
    let opt_rep = if args.contains_id("rep") {
        let rep = args.get_one::<String>("rep").unwrap();
        let pos = rep
            .find('=')
            .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{rep}`"))
            .unwrap();
        (rep[..pos].to_string(), rep[pos + 1..].to_string())
    } else {
        (String::new(), String::new())
    };

    //----------------------------
    // Ops
    //----------------------------
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

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

    if !opt_strain.as_os_str().is_empty() {
        info!("==> Loading `{}` to `rank` and `asm`", opt_strain.display());
        // strain, rank
        let dmp = std::fs::File::open(opt_strain)?;
        insert_strain(&dmp, &conn)?;
    }

    if !opt_size.as_os_str().is_empty() {
        info!("==> Loading `{}` to `seq`", opt_size.display());
        // sequence name, size
        let dmp = std::fs::File::open(opt_size)?;
        insert_size(&dmp, &conn)?;
    }

    if !opt_clust.as_os_str().is_empty() {
        info!(
            "==> Loading `{}` to `rep` and `rep_seq`",
            opt_clust.display()
        );
        // rep, seq
        let dmp = std::fs::File::open(opt_clust)?;
        insert_clust(&dmp, &conn)?;
    }

    if !opt_anno.as_os_str().is_empty() {
        info!("==> Loading `{}` to `seq`", opt_anno.display());
        // rep, seq
        let dmp = std::fs::File::open(opt_anno)?;
        insert_anno(&dmp, &conn)?;
    }

    if !opt_asmseq.as_os_str().is_empty() {
        info!("==> Loading `{}` to `asm_seq`", opt_asmseq.display());
        // sequence name, assembly name
        let dmp = std::fs::File::open(opt_asmseq)?;
        insert_asmseq(&dmp, &conn)?;
    }

    if !opt_rep.1.is_empty() {
        info!("==> Loading `{}` to `rep.{}`", opt_rep.1, opt_rep.0);
        // family, rep
        let dmp = std::fs::File::open(opt_rep.1)?;
        insert_rep(&dmp, &opt_rep.0, &conn)?;
    }

    Ok(())
}

fn insert_strain(
    dmp: &std::fs::File,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut rank_stmt = conn.prepare("INSERT OR IGNORE INTO rank(name) VALUES (?1)")?;

    let mut asm_stmt = conn.prepare(
        "INSERT INTO asm(name, rank_id) VALUES (?1, (SELECT id FROM rank WHERE name = ?2))"
    )?;

    conn.execute_batch("BEGIN;")?;
    for result in tsv_rdr.records() {
        let record = result?;
        let strain: String = record[0].trim().parse()?;
        let rank: String = record[1].trim().parse()?;

        rank_stmt.execute([&rank])?;
        asm_stmt.execute(rusqlite::params![&strain, &rank])?;
    }
    conn.execute_batch("COMMIT;")?;
    Ok(())
}

fn insert_size(dmp: &std::fs::File, conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmt =
        conn.prepare("INSERT OR IGNORE INTO seq(name, size) VALUES (?1, ?2)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        let name: String = record[0].trim().parse()?;
        let size: i64 = record[1].trim().parse()?;

        stmt.execute(rusqlite::params![&name, size])?;

        if i > 0 && i % 10000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

fn insert_clust(dmp: &std::fs::File, conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut rep_stmt = conn.prepare("INSERT OR IGNORE INTO rep(name) VALUES (?1)")?;

    let mut rep_seq_stmt = conn.prepare(
        "INSERT INTO rep_seq(rep_id, seq_id) VALUES (
            (SELECT id FROM rep WHERE name = ?1),
            (SELECT id FROM seq WHERE name = ?2)
        )",
    )?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        let rep: String = record[0].trim().parse()?;
        let seq: String = record[1].trim().parse()?;

        rep_stmt.execute([&rep])?;
        rep_seq_stmt.execute(rusqlite::params![&rep, &seq])?;

        if i > 0 && i % 10000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

fn insert_anno(dmp: &std::fs::File, conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmt = conn.prepare("UPDATE seq SET anno = ?1 WHERE seq.name = ?2")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        let name: String = record[0].trim().parse()?;
        let anno: String = record[1].trim().parse()?;

        stmt.execute(rusqlite::params![&anno, &name])?;

        if i > 0 && i % 10000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

fn insert_asmseq(
    dmp: &std::fs::File,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmt = conn.prepare(
        "INSERT INTO asm_seq(asm_id, seq_id) VALUES (
            (SELECT id FROM asm WHERE name = ?1),
            (SELECT id FROM seq WHERE name = ?2)
        )",
    )?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;

        // sequence name, assembly name
        let seq: String = record[0].trim().parse()?;
        let asm: String = record[1].trim().parse()?;

        stmt.execute(rusqlite::params![&asm, &seq])?;

        if i > 0 && i % 10000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

fn insert_rep(
    dmp: &std::fs::File,
    field: &str,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    // Validate field name to prevent SQL injection
    let field = validate_rep_field(field)?;

    // empty field before updating
    conn.execute_batch(&format!(
        "
        UPDATE rep
        SET {} = NULL;
        ",
        field
    ))?;

    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    // Use parameterized query with dynamic column name
    let sql = format!("UPDATE rep SET {} = ?1 WHERE rep.name = ?2", field);
    let mut stmt = conn.prepare(&sql)?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        let family: String = record[0].trim().parse()?;
        let rep: String = record[1].trim().parse()?;

        stmt.execute(rusqlite::params![&family, &rep])?;

        if i > 0 && i % 10000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

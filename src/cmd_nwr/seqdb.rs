use clap::*;
use itertools::Itertools;
use log::info;
use simplelog::*;
use std::path::PathBuf;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("seqdb")
        .about("Init the seq database")
        .after_help(format!(
            r###"
In RefSeq, many species contain hundreds or thousands of assemblies where many of
the protein sequences are identical or highly similar

./seq.sqlite

* This database is a repository of protein sequence information per rank group

* If `--strain` is called without specifying a path, it will load the default file under `--dir`

* `--rep` requires key-value pair, `--rep f1=file`

* The DDL

{}
"###,
            DDL_SEQ.lines().map(|l| format!("    {}", l)).join("\n")
        ))
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .default_value(".")
                .help("Change working directory"),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .action(ArgAction::SetTrue)
                .help("Init (delete) the db"),
        )
        .arg(
            Arg::new("strain")
                .long("strain")
                .num_args(0..=1)
                .help("Load strains.tsv"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .num_args(0..=1)
                .help("Load sizes.tsv"),
        )
        .arg(
            Arg::new("clust")
                .long("clust")
                .num_args(0..=1)
                .help("Load rep_cluster.tsv"),
        )
        .arg(
            Arg::new("anno")
                .long("anno")
                .num_args(0..=1)
                .help("Load anno.tsv"),
        )
        .arg(
            Arg::new("asmseq")
                .long("asmseq")
                .num_args(0..=1)
                .help("Load asmseq.tsv"),
        )
        .arg(
            Arg::new("rep")
                .long("rep")
                .num_args(1)
                .help("Load features into rep"),
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
-- Regular indecies
CREATE INDEX rep_idx_f1 ON rep(f1);
CREATE INDEX rep_idx_f2 ON rep(f2);
CREATE INDEX rep_idx_f3 ON rep(f3);
CREATE INDEX rep_idx_f4 ON rep(f4);
CREATE INDEX rep_idx_f5 ON rep(f5);
CREATE INDEX rep_idx_f6 ON rep(f6);
CREATE INDEX rep_idx_f7 ON rep(f7);
CREATE INDEX rep_idx_f8 ON rep(f8);
-- Case-insensitive indecies for `like`
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
        // sequence name, asm
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

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];

    for result in tsv_rdr.records() {
        let record = result?;
        let strain: String = record[0].trim().parse()?;
        let rank: String = record[1].trim().parse()?;

        stmts.push(format!(
            "
            INSERT OR IGNORE INTO rank(name)
            VALUES ('{}');
            ",
            rank
        ));
        stmts.push(format!(
            "
            INSERT INTO asm(name, rank_id)
            VALUES (
                '{}',
                (SELECT id FROM rank WHERE name = '{}')
            );
            ",
            strain, rank
        ));
    }

    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;
    Ok(())
}

fn insert_size(dmp: &std::fs::File, conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        nwr::batch_exec(conn, &mut stmts, i)?;

        let record = result?;
        let name: String = record[0].trim().parse()?;
        let size: i64 = record[1].trim().parse()?;

        stmts.push(format!(
            "
            INSERT OR IGNORE INTO seq(name, size)
            VALUES ('{}', {});
            ",
            name, size
        ));
    }

    // Records may be left in stmts
    nwr::batch_exec(conn, &mut stmts, usize::MAX)?;

    Ok(())
}

fn insert_clust(dmp: &std::fs::File, conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        nwr::batch_exec(conn, &mut stmts, i)?;

        let record = result?;
        let rep: String = record[0].trim().parse()?;
        let seq: String = record[1].trim().parse()?;

        stmts.push(format!(
            "
            INSERT OR IGNORE INTO rep(name)
            VALUES ('{}');
            ",
            rep
        ));

        stmts.push(format!(
            "
            INSERT INTO rep_seq(rep_id, seq_id)
            VALUES (
                (SELECT id FROM rep WHERE name = '{}'),
                (SELECT id FROM seq WHERE name = '{}')
            );
            ",
            rep, seq
        ));
    }

    // Records may be left in stmts
    nwr::batch_exec(conn, &mut stmts, usize::MAX)?;

    Ok(())
}

fn insert_anno(dmp: &std::fs::File, conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        nwr::batch_exec(conn, &mut stmts, i)?;

        let record = result?;
        let name: String = record[0].trim().parse()?;
        let anno: String = record[1].trim().parse()?;

        stmts.push(format!(
            "
            UPDATE seq
            SET anno = '{}'
            WHERE seq.name = '{}';
            ",
            anno, name
        ));
    }

    // Records may be left in stmts
    nwr::batch_exec(conn, &mut stmts, usize::MAX)?;

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

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        nwr::batch_exec(conn, &mut stmts, i)?;

        let record = result?;

        // sequence name, assembly name
        let seq: String = record[0].trim().parse()?;
        let asm: String = record[1].trim().parse()?;

        stmts.push(format!(
            "
            INSERT INTO asm_seq(asm_id, seq_id)
            VALUES (
                (SELECT id FROM asm WHERE name = '{}'),
                (SELECT id FROM seq WHERE name = '{}')
            );
            ",
            asm, seq
        ));
    }

    // Records may be left in stmts
    nwr::batch_exec(conn, &mut stmts, usize::MAX)?;

    Ok(())
}

fn insert_rep(
    dmp: &std::fs::File,
    field: &str,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
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

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        nwr::batch_exec(conn, &mut stmts, i)?;

        let record = result?;
        let family: String = record[0].trim().parse()?;
        let rep: String = record[1].trim().parse()?;

        stmts.push(format!(
            "
            UPDATE rep
            SET {} = '{}'
            WHERE rep.name = '{}';
            ",
            field, family, rep
        ));
    }
    // Records may be left in stmts
    nwr::batch_exec(conn, &mut stmts, usize::MAX)?;

    Ok(())
}

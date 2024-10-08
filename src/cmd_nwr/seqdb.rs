use clap::*;
use itertools::Itertools;
use log::{debug, info};
use rusqlite::Connection;
use simplelog::*;
use std::fs::File;

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

* The DDL

{}
"###,
            DDL_SEQ.lines().map(|l| format!("    {}", l)).join("\n")
        ))
        .arg(
            Arg::new("init")
                .long("init")
                .action(ArgAction::SetTrue)
                .help("Init (delete) the db"),
        )
        .arg(
            Arg::new("strain")
                .long("strain")
                .action(ArgAction::SetTrue)
                .help("Load strain.tsv"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .action(ArgAction::SetTrue)
                .help("Load size.tsv"),
        )
        .arg(
            Arg::new("anno")
                .long("anno")
                .action(ArgAction::SetTrue)
                .help("Load anno.tsv"),
        )
        .arg(
            Arg::new("clust")
                .long("clust")
                .action(ArgAction::SetTrue)
                .help("Load clust.tsv"),
        )
        .arg(
            Arg::new("seq")
                .long("seq")
                .action(ArgAction::SetTrue)
                .help("Load seq.tsv"),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .default_value(".")
                .help("Change working directory"),
        )
}

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
    annotation TEXT
);
-- representative
CREATE TABLE rep (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    f1 TEXT,
    f2 TEXT,
    f3 TEXT
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
"###;

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let is_init = args.get_flag("init");
    let is_strain = args.get_flag("strain");
    let is_size = args.get_flag("size");
    let is_anno = args.get_flag("anno");
    let is_clust = args.get_flag("clust");
    let is_seq = args.get_flag("seq");

    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

    let dir = std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf();
    let db = dir.join("seq.sqlite");
    if is_init && db.exists() {
        std::fs::remove_file(&db).unwrap();
    }

    info!("==> Opening database");
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

    if is_strain {
        info!("==> Loading strains.tsv to `rank` and `asm`");
        // strain, rank
        let dmp = File::open(dir.join("strains.tsv"))?;
        insert_strain(&dmp, &conn)?;
    }

    if is_size {
        info!("==> Loading size.tsv to `seq`");
        // sequence name, size
        let dmp = File::open(dir.join("size.tsv"))?;
        insert_size(&dmp, &conn)?;
    }

    if is_anno {
        info!("==> Loading anno.tsv to `seq`");
        // sequence name, anno
        let dmp = File::open(dir.join("anno.tsv"))?;
        insert_anno(&dmp, &conn)?;
    }

    if is_clust {
        info!("==> Loading res_cluster.tsv to `rep` and `rep_seq`");
        // rep, seq
        let dmp = File::open(dir.join("res_cluster.tsv"))?;
        insert_clust(&dmp, &conn)?;
    }

    if is_seq {
        info!("==> Loading seq.tsv to `asm_seq`");
        // sequence name, asm
        let dmp = File::open(dir.join("seq.tsv"))?;
        insert_seq(&dmp, conn)?;
    }

    Ok(())
}

fn insert_strain(dmp: &File, conn: &Connection) -> anyhow::Result<()> {
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

fn insert_size(dmp: &File, conn: &Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        batch_exec(&conn, &mut stmts, i)?;

        let record = result?;
        let name: String = record[0].trim().parse()?;
        let size: i64 = record[1].trim().parse()?;

        stmts.push(format!(
            "
            INSERT INTO seq(name, size)
            VALUES ('{}', {});
            ",
            name, size
        ));
    }

    // Records may be left in stmts
    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;
    Ok(())
}

fn insert_anno(dmp: &File, conn: &Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        batch_exec(&conn, &mut stmts, i)?;

        let record = result?;
        let name: String = record[0].trim().parse()?;
        let anno: String = record[1].trim().parse()?;

        stmts.push(format!(
            "
            UPDATE seq
            SET annotation = '{}'
            WHERE seq.name = '{}';
            ",
            anno, name
        ));
    }

    // Records may be left in stmts
    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;
    Ok(())
}

fn insert_clust(dmp: &File, conn: &Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        batch_exec(&conn, &mut stmts, i)?;

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
    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;

    Ok(())
}

fn insert_seq(dmp: &File, conn: Connection) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, result) in tsv_rdr.records().enumerate() {
        batch_exec(&conn, &mut stmts, i)?;

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
    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;
    Ok(())
}

fn batch_exec(
    conn: &Connection,
    stmts: &mut Vec<String>,
    i: usize,
) -> anyhow::Result<()> {
    if i > 1 && i % 1000 == 0 {
        stmts.push(String::from("COMMIT;"));
        let stmt = &stmts.join("\n");
        conn.execute_batch(stmt)?;
        stmts.clear();
        stmts.push(String::from("BEGIN;"));
    }
    if i > 1 && i % 10000 == 0 {
        debug!("Read {} records", i);
    }
    Ok(())
}

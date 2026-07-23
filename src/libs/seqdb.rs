use std::fs::File;
use std::path::Path;

/// Valid field names for the rep table
pub const VALID_REP_FIELDS: &[&str] = &["f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8"];

/// Return the static SQL used to clear a rep field.
pub fn rep_clear_sql(field: &str) -> anyhow::Result<&'static str> {
    let sql = match field {
        "f1" => "UPDATE rep SET f1 = NULL;",
        "f2" => "UPDATE rep SET f2 = NULL;",
        "f3" => "UPDATE rep SET f3 = NULL;",
        "f4" => "UPDATE rep SET f4 = NULL;",
        "f5" => "UPDATE rep SET f5 = NULL;",
        "f6" => "UPDATE rep SET f6 = NULL;",
        "f7" => "UPDATE rep SET f7 = NULL;",
        "f8" => "UPDATE rep SET f8 = NULL;",
        _ => {
            anyhow::bail!(
                "Invalid rep field '{field}'. Valid fields are: {VALID_REP_FIELDS:?}"
            )
        }
    };
    Ok(sql)
}

/// Return the static SQL used to update a rep field.
pub fn rep_update_sql(field: &str) -> anyhow::Result<&'static str> {
    let sql = match field {
        "f1" => "UPDATE rep SET f1 = ?1 WHERE rep.name = ?2",
        "f2" => "UPDATE rep SET f2 = ?1 WHERE rep.name = ?2",
        "f3" => "UPDATE rep SET f3 = ?1 WHERE rep.name = ?2",
        "f4" => "UPDATE rep SET f4 = ?1 WHERE rep.name = ?2",
        "f5" => "UPDATE rep SET f5 = ?1 WHERE rep.name = ?2",
        "f6" => "UPDATE rep SET f6 = ?1 WHERE rep.name = ?2",
        "f7" => "UPDATE rep SET f7 = ?1 WHERE rep.name = ?2",
        "f8" => "UPDATE rep SET f8 = ?1 WHERE rep.name = ?2",
        _ => {
            anyhow::bail!(
                "Invalid rep field '{field}'. Valid fields are: {VALID_REP_FIELDS:?}"
            )
        }
    };
    Ok(sql)
}

/// Ensure a CSV record has at least `min` fields, returning a descriptive
/// error referencing the 1-based line number and file path.
fn require_min_fields(
    record: &csv::StringRecord,
    min: usize,
    i: usize,
    path: &Path,
) -> anyhow::Result<()> {
    if record.len() < min {
        anyhow::bail!(
            "Line {} in {} has fewer than {} columns",
            i + 1,
            path.display(),
            min
        );
    }
    Ok(())
}

/// Run a `SELECT EXISTS(...)` check via `stmt` for `name` and bail with a
/// uniform error if the name is absent from the target table.
fn ensure_exists(
    stmt: &mut rusqlite::Statement,
    name: &str,
    table: &str,
    i: usize,
    path: &Path,
) -> anyhow::Result<()> {
    let exists: bool = stmt.query_row(rusqlite::params![name], |row| row.get(0))?;
    if !exists {
        anyhow::bail!(
            "Line {} in {}: {} name '{}' not found in {} table",
            i + 1,
            path.display(),
            table,
            name,
            table
        );
    }
    Ok(())
}

/// DDL for the seq `SQLite` database.
// https://stackoverflow.com/questions/58684279/can-an-index-on-a-text-column-speed-up-prefix-based-like-queries
pub static DDL_SEQ: &str = r"
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

";

/// Load strains and ranks into `rank` and `asm`.
pub fn insert_strain(
    dmp: &File,
    path: &std::path::Path,
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
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        require_min_fields(&record, 2, i, path)?;
        let strain: String = record[0].trim().to_string();
        let rank: String = record[1].trim().to_string();

        rank_stmt.execute([&rank])?;
        asm_stmt.execute(rusqlite::params![&strain, &rank])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;
    Ok(())
}

/// Load sequence sizes into `seq`.
pub fn insert_size(
    dmp: &File,
    path: &std::path::Path,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmt =
        conn.prepare("INSERT OR IGNORE INTO seq(name, size) VALUES (?1, ?2)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        require_min_fields(&record, 2, i, path)?;
        let name: String = record[0].trim().to_string();
        let size: i64 = record[1].trim().parse().map_err(|e| {
            anyhow::anyhow!(
                "Invalid size at line {} in {}: {}",
                i + 1,
                path.display(),
                e
            )
        })?;

        stmt.execute(rusqlite::params![&name, size])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

/// Load rep/seq clusters into `rep` and `rep_seq`.
pub fn insert_clust(
    dmp: &File,
    path: &std::path::Path,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
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

    let mut seq_exists =
        conn.prepare("SELECT EXISTS(SELECT 1 FROM seq WHERE name = ?1)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        require_min_fields(&record, 2, i, path)?;
        let rep: String = record[0].trim().to_string();
        let seq: String = record[1].trim().to_string();

        ensure_exists(&mut seq_exists, &seq, "seq", i, path)?;

        rep_stmt.execute([&rep])?;
        rep_seq_stmt.execute(rusqlite::params![&rep, &seq])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

/// Load annotations into `seq`.
pub fn insert_anno(
    dmp: &File,
    path: &std::path::Path,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmt = conn.prepare("UPDATE seq SET anno = ?1 WHERE seq.name = ?2")?;
    let mut seq_exists =
        conn.prepare("SELECT EXISTS(SELECT 1 FROM seq WHERE name = ?1)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        require_min_fields(&record, 2, i, path)?;
        let name: String = record[0].trim().to_string();
        let anno: String = record[1].trim().to_string();

        ensure_exists(&mut seq_exists, &name, "seq", i, path)?;

        stmt.execute(rusqlite::params![&anno, &name])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

/// Load assembly/sequence associations into `asm_seq`.
pub fn insert_asmseq(
    dmp: &File,
    path: &std::path::Path,
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

    let mut seq_exists =
        conn.prepare("SELECT EXISTS(SELECT 1 FROM seq WHERE name = ?1)")?;
    let mut asm_exists =
        conn.prepare("SELECT EXISTS(SELECT 1 FROM asm WHERE name = ?1)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        require_min_fields(&record, 2, i, path)?;

        // sequence name, assembly name
        let seq: String = record[0].trim().to_string();
        let asm: String = record[1].trim().to_string();

        ensure_exists(&mut seq_exists, &seq, "seq", i, path)?;
        ensure_exists(&mut asm_exists, &asm, "asm", i, path)?;

        stmt.execute(rusqlite::params![&asm, &seq])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

/// Load a rep field from a two-column TSV into `rep`.
pub fn insert_rep(
    dmp: &File,
    field: &str,
    path: &std::path::Path,
    conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    let mut tsv_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(dmp);

    let mut stmt = conn.prepare(rep_update_sql(field)?)?;
    let mut rep_exists =
        conn.prepare("SELECT EXISTS(SELECT 1 FROM rep WHERE name = ?1)")?;

    conn.execute_batch("BEGIN;")?;
    // Empty the field before updating so that the clear and the following
    // updates are atomic.
    conn.execute_batch(rep_clear_sql(field)?)?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        require_min_fields(&record, 2, i, path)?;
        let value: String = record[0].trim().to_string();
        let rep: String = record[1].trim().to_string();

        ensure_exists(&mut rep_exists, &rep, "rep", i, path)?;

        stmt.execute(rusqlite::params![&value, &rep])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

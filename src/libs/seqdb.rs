use std::collections::HashMap;
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

/// Returns `true` if every field in the record is empty or whitespace only.
///
/// The `csv` crate parses a completely blank line as a record with one empty
/// field, so callers should skip such records rather than treating them as
/// malformed data.
fn is_blank_record(record: &csv::StringRecord) -> bool {
    record.iter().all(|f| f.trim().is_empty())
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

    // Cache rank IDs to avoid a per-row subquery on the rank table.
    let mut rank_cache: HashMap<String, i64> = HashMap::new();
    let mut load_ranks = conn.prepare("SELECT id, name FROM rank")?;
    let mut rows = load_ranks.query([])?;
    while let Some(row) = rows.next()? {
        rank_cache.insert(row.get(1)?, row.get(0)?);
    }

    let mut rank_insert = conn.prepare("INSERT INTO rank(name) VALUES (?1)")?;
    let mut asm_stmt = conn.prepare("INSERT INTO asm(name, rank_id) VALUES (?1, ?2)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        if is_blank_record(&record) {
            continue;
        }
        require_min_fields(&record, 2, i, path)?;
        let strain: String = record[0].trim().to_string();
        let rank: String = record[1].trim().to_string();

        let rank_id = match rank_cache.get(&rank) {
            Some(&id) => id,
            None => {
                rank_insert.execute([&rank])?;
                let id = conn.last_insert_rowid();
                rank_cache.insert(rank.clone(), id);
                id
            }
        };

        asm_stmt.execute(rusqlite::params![&strain, rank_id])?;

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
        if is_blank_record(&record) {
            continue;
        }
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

    // Cache seq IDs upfront; build rep IDs incrementally to avoid per-row subqueries.
    let mut seq_cache: HashMap<String, i64> = HashMap::new();
    let mut load_seq = conn.prepare("SELECT id, name FROM seq")?;
    let mut rows = load_seq.query([])?;
    while let Some(row) = rows.next()? {
        seq_cache.insert(row.get(1)?, row.get(0)?);
    }

    let mut rep_cache: HashMap<String, i64> = HashMap::new();
    let mut rep_insert = conn.prepare("INSERT INTO rep(name) VALUES (?1)")?;
    let mut rep_seq_stmt =
        conn.prepare("INSERT INTO rep_seq(rep_id, seq_id) VALUES (?1, ?2)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        if is_blank_record(&record) {
            continue;
        }
        require_min_fields(&record, 2, i, path)?;
        let rep: String = record[0].trim().to_string();
        let seq: String = record[1].trim().to_string();

        let seq_id = seq_cache.get(&seq).copied().ok_or_else(|| {
            anyhow::anyhow!(
                "Line {} in {}: seq name '{}' not found in seq table",
                i + 1,
                path.display(),
                seq
            )
        })?;

        let rep_id = match rep_cache.get(&rep) {
            Some(&id) => id,
            None => {
                rep_insert.execute([&rep])?;
                let id = conn.last_insert_rowid();
                rep_cache.insert(rep.clone(), id);
                id
            }
        };

        rep_seq_stmt.execute(rusqlite::params![rep_id, seq_id])?;

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

    // Cache seq IDs to avoid a per-row EXISTS lookup.
    let mut seq_cache: HashMap<String, i64> = HashMap::new();
    let mut load_seq = conn.prepare("SELECT id, name FROM seq")?;
    let mut rows = load_seq.query([])?;
    while let Some(row) = rows.next()? {
        seq_cache.insert(row.get(1)?, row.get(0)?);
    }

    let mut stmt = conn.prepare("UPDATE seq SET anno = ?1 WHERE seq.name = ?2")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        if is_blank_record(&record) {
            continue;
        }
        require_min_fields(&record, 2, i, path)?;
        let name: String = record[0].trim().to_string();
        let anno: String = record[1].trim().to_string();

        if !seq_cache.contains_key(&name) {
            anyhow::bail!(
                "Line {} in {}: seq name '{}' not found in seq table",
                i + 1,
                path.display(),
                name
            );
        }

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

    // Cache asm and seq IDs upfront to avoid per-row subqueries.
    let mut asm_cache: HashMap<String, i64> = HashMap::new();
    let mut load_asm = conn.prepare("SELECT id, name FROM asm")?;
    let mut rows = load_asm.query([])?;
    while let Some(row) = rows.next()? {
        asm_cache.insert(row.get(1)?, row.get(0)?);
    }

    let mut seq_cache: HashMap<String, i64> = HashMap::new();
    let mut load_seq = conn.prepare("SELECT id, name FROM seq")?;
    let mut rows = load_seq.query([])?;
    while let Some(row) = rows.next()? {
        seq_cache.insert(row.get(1)?, row.get(0)?);
    }

    let mut stmt =
        conn.prepare("INSERT INTO asm_seq(asm_id, seq_id) VALUES (?1, ?2)")?;

    conn.execute_batch("BEGIN;")?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        if is_blank_record(&record) {
            continue;
        }
        require_min_fields(&record, 2, i, path)?;

        // sequence name, assembly name
        let seq: String = record[0].trim().to_string();
        let asm: String = record[1].trim().to_string();

        let seq_id = seq_cache.get(&seq).copied().ok_or_else(|| {
            anyhow::anyhow!(
                "Line {} in {}: seq name '{}' not found in seq table",
                i + 1,
                path.display(),
                seq
            )
        })?;
        let asm_id = asm_cache.get(&asm).copied().ok_or_else(|| {
            anyhow::anyhow!(
                "Line {} in {}: asm name '{}' not found in asm table",
                i + 1,
                path.display(),
                asm
            )
        })?;

        stmt.execute(rusqlite::params![asm_id, seq_id])?;

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

    // Cache rep IDs to avoid a per-row EXISTS lookup.
    let mut rep_cache: HashMap<String, i64> = HashMap::new();
    let mut load_rep = conn.prepare("SELECT id, name FROM rep")?;
    let mut rows = load_rep.query([])?;
    while let Some(row) = rows.next()? {
        rep_cache.insert(row.get(1)?, row.get(0)?);
    }

    let mut stmt = conn.prepare(rep_update_sql(field)?)?;

    conn.execute_batch("BEGIN;")?;
    // Empty the field before updating so that the clear and the following
    // updates are atomic.
    conn.execute_batch(rep_clear_sql(field)?)?;
    for (i, result) in tsv_rdr.records().enumerate() {
        let record = result?;
        if is_blank_record(&record) {
            continue;
        }
        require_min_fields(&record, 2, i, path)?;
        let value: String = record[0].trim().to_string();
        let rep: String = record[1].trim().to_string();

        if !rep_cache.contains_key(&rep) {
            anyhow::bail!(
                "Line {} in {}: rep name '{}' not found in rep table",
                i + 1,
                path.display(),
                rep
            );
        }

        stmt.execute(rusqlite::params![&value, &rep])?;

        crate::libs::io::progress_dot(i)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    Ok(())
}

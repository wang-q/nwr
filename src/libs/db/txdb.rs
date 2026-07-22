use log::{debug, info};
use std::fs::File;

/// DDL for the NCBI taxonomy SQLite database.
static DDL_TX: &str = r"
DROP TABLE IF EXISTS division;
DROP TABLE IF EXISTS node;
DROP TABLE IF EXISTS name;

CREATE TABLE division (
    id       INTEGER      NOT NULL
                          PRIMARY KEY,
    division VARCHAR (50) NOT NULL
);

CREATE TABLE node (
    tax_id        INTEGER      NOT NULL
                               PRIMARY KEY,
    parent_tax_id INTEGER,
    rank          VARCHAR (25) NOT NULL,
    division_id   INTEGER      NOT NULL,
    comment       TEXT,
    FOREIGN KEY (
        division_id
    )
    REFERENCES division (id)
);

CREATE TABLE name (
    id         INTEGER      NOT NULL
                            PRIMARY KEY,
    tax_id     INTEGER      NOT NULL,
    name       VARCHAR (50) NOT NULL,
    name_class VARCHAR (50) NOT NULL
);
";

/// Build the taxonomy database from NCBI `division.dmp`, `names.dmp` and `nodes.dmp`.
///
/// `nwrdir` is the directory containing the NCBI taxonomy dump files. The
/// resulting database is written to `nwrdir/taxonomy.sqlite`.
pub fn run(nwrdir: &std::path::Path) -> anyhow::Result<()> {
    let file = nwrdir.join("taxonomy.sqlite");
    if file.exists() {
        std::fs::remove_file(&file)?;
    }

    info!("==> Opening database");
    let conn = rusqlite::Connection::open(file)?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = OFF;
        PRAGMA synchronous = 0;
        PRAGMA cache_size = 1000000;
        PRAGMA locking_mode = EXCLUSIVE;
        PRAGMA temp_store = MEMORY;
        ",
    )?;

    info!("==> Create tables");
    conn.execute_batch(DDL_TX)?;

    // divisions
    info!("==> Loading division.dmp");
    {
        let dmp = File::open(nwrdir.join("division.dmp"))?;
        let mut tsv_rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'|')
            .from_reader(dmp);

        let mut stmt =
            conn.prepare("INSERT INTO division (id, division) VALUES (?1, ?2)")?;

        // Intentionally use explicit SQL BEGIN/COMMIT rather than rusqlite::Transaction.
        conn.execute_batch("BEGIN;")?;
        for (i, result) in tsv_rdr.records().enumerate() {
            let record = result?;
            if record.len() < 3 {
                return Err(anyhow::anyhow!(
                    "division.dmp record has {} fields, expected at least 3: {:?}",
                    record.len(),
                    record
                ));
            }
            let id: i64 = record[0].trim().parse().map_err(|e| {
                anyhow::anyhow!("Invalid id at line {} in division.dmp: {}", i + 1, e)
            })?;
            let name: String = record[2].trim().to_string();
            stmt.execute(rusqlite::params![id, name])?;
        }
        conn.execute_batch("COMMIT;")?;

        debug!("Done inserting divisions");
    }

    // names
    info!("==> Loading names.dmp");
    {
        let dmp = File::open(nwrdir.join("names.dmp"))?;
        let mut tsv_rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'|')
            .from_reader(dmp);

        let mut stmt = conn.prepare(
            "INSERT INTO name (tax_id, name, name_class) VALUES (?1, ?2, ?3)",
        )?;

        // Intentionally use explicit SQL BEGIN/COMMIT rather than rusqlite::Transaction.
        conn.execute_batch("BEGIN;")?;
        for (i, result) in tsv_rdr.records().enumerate() {
            let record = result?;
            if record.len() < 4 {
                return Err(anyhow::anyhow!(
                    "names.dmp record has {} fields, expected at least 4: {:?}",
                    record.len(),
                    record
                ));
            }

            // tax_id, name, unique_name, name_class
            let tax_id: i64 = record[0].trim().parse().map_err(|e| {
                anyhow::anyhow!("Invalid tax_id at line {} in names.dmp: {}", i + 1, e)
            })?;
            let name: String = record[1].trim().to_string();
            let name_class: String = record[3].trim().to_string();

            stmt.execute(rusqlite::params![tax_id, name, name_class])?;

            crate::libs::io::progress_dot(i)?;
        }
        eprintln!();
        conn.execute_batch("COMMIT;")?;

        debug!("Creating indexes for name");
        conn.execute("CREATE INDEX idx_name_tax_id ON name(tax_id);", [])?;
        conn.execute("CREATE INDEX idx_name_name ON name(name);", [])?;
    }

    // nodes
    info!("==> Loading nodes.dmp");
    {
        let dmp = File::open(nwrdir.join("nodes.dmp"))?;
        let mut tsv_rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'|')
            .from_reader(dmp);

        let mut stmt = conn.prepare(
            "INSERT INTO node (tax_id, parent_tax_id, rank, division_id, comment) VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;

        // Intentionally use explicit SQL BEGIN/COMMIT rather than rusqlite::Transaction.
        conn.execute_batch("BEGIN;")?;
        for (i, result) in tsv_rdr.records().enumerate() {
            let record = result?;
            if record.len() < 13 {
                return Err(anyhow::anyhow!(
                    "nodes.dmp record has {} fields, expected at least 13: {:?}",
                    record.len(),
                    record
                ));
            }

            // tax_id, parent, rank, code, divid, undef, gen_code, undef, mito
            let tax_id: i64 = record[0].trim().parse().map_err(|e| {
                anyhow::anyhow!("Invalid tax_id at line {} in nodes.dmp: {}", i + 1, e)
            })?;
            let parent_tax_id: i64 = record[1].trim().parse().map_err(|e| {
                anyhow::anyhow!(
                    "Invalid parent_tax_id at line {} in nodes.dmp: {}",
                    i + 1,
                    e
                )
            })?;
            let rank: String = record[2].trim().to_string();
            let division_id: i64 = record[4].trim().parse().map_err(|e| {
                anyhow::anyhow!(
                    "Invalid division_id at line {} in nodes.dmp: {}",
                    i + 1,
                    e
                )
            })?;
            let comments: String = record[12].trim().to_string();

            stmt.execute(rusqlite::params![
                tax_id,
                parent_tax_id,
                rank,
                division_id,
                comments
            ])?;

            crate::libs::io::progress_dot(i)?;
        }
        eprintln!();
        conn.execute_batch("COMMIT;")?;

        debug!("Creating indexes for node");
        conn.execute(
            "CREATE INDEX idx_node_parent_id ON node(parent_tax_id);",
            [],
        )?;
    }

    Ok(())
}

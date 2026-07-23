/// Apply PRAGMA settings tuned for bulk import into a fresh SQLite database.
///
/// Disables journaling and synchronous writes, increases the cache size, and
/// keeps temporary data in memory. This should only be used when the database
/// is not shared with other connections.
pub fn apply_import_pragmas(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = OFF;
        PRAGMA synchronous = 0;
        PRAGMA cache_size = 1000000;
        PRAGMA locking_mode = EXCLUSIVE;
        PRAGMA temp_store = MEMORY;
        ",
    )?;
    Ok(())
}

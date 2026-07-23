use anyhow::Context;
use std::collections::HashMap;
use std::path::Path;

/// Chunk size for `SQLite` `IN (...)` placeholder limits in [`get_taxon`].
const CHUNK_SIZE: usize = 900;

/// A single NCBI taxonomy node with its names and lineage metadata.
#[derive(Debug, Clone, Default)]
pub struct Taxon {
    /// NCBI taxon ID.
    pub tax_id: i64,
    /// Parent taxon ID.
    pub parent_tax_id: i64,
    /// Taxonomic rank (e.g. species, genus).
    pub rank: String,
    /// NCBI division name.
    pub division: String,
    /// Map of name classes to their values (scientific names, synonyms, etc.).
    pub names: HashMap<String, Vec<String>>,
    /// Optional NCBI comments for this taxon.
    pub comments: Option<String>,
}

impl Taxon {
    /// Returns the first scientific name associated with this taxon, if any.
    #[must_use]
    pub fn scientific_name(&self) -> Option<&str> {
        self.names
            .get("scientific name")
            .and_then(|v| v.first())
            .map(std::string::String::as_str)
    }
}

impl std::fmt::Display for Taxon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let sciname = self.scientific_name().unwrap_or("Unknown");
        let l1 = format!("{} - {}", sciname, self.rank);
        writeln!(f, "{l1}")?;
        write!(f, "{}", "-".repeat(l1.chars().count()))?;
        writeln!(f, "\nNCBI Taxonomy ID: {}", self.tax_id)?;

        if let Some(synonyms) = self.names.get("synonym") {
            writeln!(f, "Same as:")?;
            for synonym in synonyms {
                writeln!(f, "* {synonym}")?;
            }
        }

        if let Some(genbank_names) = self.names.get("genbank common name") {
            if let Some(genbank) = genbank_names.first() {
                writeln!(f, "Commonly named {genbank}.")?;
            }
        }

        if let Some(common_names) = self.names.get("common name") {
            writeln!(f, "Also known as:")?;
            for name in common_names {
                writeln!(f, "* {name}")?;
            }
        }

        if let Some(authorities) = self.names.get("authority") {
            writeln!(f, "First description:")?;
            for authority in authorities {
                writeln!(f, "* {authority}")?;
            }
        }

        writeln!(f, "Part of the {}.", self.division)?;

        if let Some(ref comments) = self.comments {
            writeln!(f, "\nComments: {comments}")?;
        }

        Ok(())
    }
}

/// nwr working path
///
/// ```
/// let path = nwr::nwr_path().unwrap();
///
/// assert!(std::path::Path::new(&path).exists());
/// ```
pub fn nwr_path() -> anyhow::Result<std::path::PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot get home directory"))?;
    let path = home.join(".nwr/");
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

/// Resolve the nwr working directory from CLI args or the default `~/.nwr`.
pub fn get_nwr_dir(
    args: &clap::ArgMatches,
    arg_name: &str,
) -> anyhow::Result<std::path::PathBuf> {
    args.get_one::<String>(arg_name)
        .map_or_else(nwr_path, |dir| Ok(Path::new(dir).to_path_buf()))
}

/// Connect taxonomy.sqlite in this dir
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// assert_eq!(conn.path().unwrap().to_str().unwrap(), "tests/nwr/taxonomy.sqlite");
/// ```
pub fn connect_txdb(dir: &Path) -> anyhow::Result<rusqlite::Connection> {
    let dbfile = dir.join("taxonomy.sqlite");
    let conn = rusqlite::Connection::open(&dbfile)
        .with_context(|| format!("failed to open {}", dbfile.display()))?;

    let table_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master
         WHERE type='table' AND name IN ('node','name','division')",
        [],
        |row| row.get(0),
    )?;
    if table_count < 3 {
        anyhow::bail!(
            "{} is missing required tables; run `nwr txdb` to initialize it",
            dbfile.display()
        );
    }

    Ok(conn)
}

/// Build fallback name candidates when an exact match fails.
///
/// Handles two common NCBI naming quirks:
/// - `sp` / `sp.` interchange (e.g. "Cladobotryum sp" -> "Cladobotryum sp.")
/// - Stripping trailing nomenclatural qualifiers (e.g. "X nom inval" -> "X",
///   matching the synonym when the scientific name is "X (nom. inval.)")
///
/// `cf.` / `aff.` are intentionally NOT handled: they denote independent
/// taxa in NCBI (verified: 0% of `cf.`/`aff.` names share a tax_id with their
/// stripped base form), so stripping them would silently mismatch the wrong
/// taxon.
fn fallback_candidates(name: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    // sp <-> sp. (only the trailing species-unspecified marker)
    if name.ends_with(" sp") && !name.ends_with(" sp.") {
        candidates.push(format!("{name}."));
    } else if name.ends_with(" sp.") {
        candidates.push(name.trim_end_matches('.').to_string());
    }

    // Strip trailing nomenclatural qualifiers in various user-typed forms.
    // NCBI canonical form is "(nom. inval.)" etc.; users often type
    // "nom inval", "nom. inval.", or "(nom. inval.)" without matching punctuation.
    const QUALIFIERS: &[&str] = &[
        " (nom. inval.)",
        " (nom. nud.)",
        " (nom. illeg.)",
        " (nom. dub.)",
        " (nom. rej.)",
        " (nom. ined.)",
        " nom. inval.",
        " nom. nud.",
        " nom. illeg.",
        " nom. dub.",
        " nom. rej.",
        " nom. ined.",
        " nom inval",
        " nom nud",
        " nom illeg",
        " nom dub",
        " nom rej",
        " nom ined",
    ];
    for suffix in QUALIFIERS {
        if let Some(base) = name.strip_suffix(suffix) {
            if !base.is_empty() {
                candidates.push(base.to_string());
            }
            break; // only one qualifier can match at the end
        }
    }

    candidates
}

/// Names to Taxonomy IDs
///
/// Resolves names via exact match against the `name` table. When an exact
/// match fails, falls back to common NCBI naming variants:
/// - `sp` / `sp.` interchange (e.g. "Cladobotryum sp" -> "Cladobotryum sp.")
/// - Stripping trailing nomenclatural qualifiers (e.g. "X nom inval" -> "X",
///   matching the synonym when the scientific name is "X (nom. inval.)")
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// let names = vec![
///     "Enterobacteria phage 933J".to_string(),
///     "Actinophage JHJ-1".to_string(),
/// ];
/// let tax_ids = nwr::get_tax_id(&conn, &names).unwrap();
///
/// assert_eq!(tax_ids, vec![12340, 12347]);
/// ```
pub fn get_tax_id(
    conn: &rusqlite::Connection,
    names: &[String],
) -> anyhow::Result<Vec<i64>> {
    if names.is_empty() {
        return Ok(vec![]);
    }

    let mut name_to_id: HashMap<String, i64> = HashMap::new();

    // 1. Exact match
    for chunk in names.chunks(CHUNK_SIZE) {
        let placeholders = (0..chunk.len()).map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "
            SELECT name, MIN(tax_id) AS tax_id
            FROM name
            WHERE name_class IN ('scientific name', 'synonym', 'genbank synonym')
            AND name IN ({placeholders})
            GROUP BY name
            "
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(chunk.iter()))?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let tax_id: i64 = row.get(1)?;
            name_to_id.insert(name, tax_id);
        }
    }

    // 2. Fallback for unresolved names: try common NCBI naming variants
    let mut fallback_queries: Vec<(String, String)> = Vec::new(); // (original, candidate)
    for name in names {
        if !name_to_id.contains_key(name) {
            for candidate in fallback_candidates(name) {
                if !name_to_id.contains_key(&candidate) {
                    fallback_queries.push((name.clone(), candidate));
                }
            }
        }
    }

    if !fallback_queries.is_empty() {
        let candidates: Vec<String> =
            fallback_queries.iter().map(|(_, c)| c.clone()).collect();
        for chunk in candidates.chunks(CHUNK_SIZE) {
            let placeholders =
                (0..chunk.len()).map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "
                SELECT name, MIN(tax_id) AS tax_id
                FROM name
                WHERE name_class IN ('scientific name', 'synonym', 'genbank synonym')
                AND name IN ({placeholders})
                GROUP BY name
                "
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(rusqlite::params_from_iter(chunk.iter()))?;
            while let Some(row) = rows.next()? {
                let candidate: String = row.get(0)?;
                let tax_id: i64 = row.get(1)?;
                name_to_id.insert(candidate, tax_id);
            }
        }

        // Map resolved candidates back to original names
        for (original, candidate) in &fallback_queries {
            if let Some(tax_id) = name_to_id.get(candidate) {
                name_to_id.insert(original.clone(), *tax_id);
            }
        }
    }

    // 3. Build result vector
    let mut tax_ids = Vec::with_capacity(names.len());
    for name in names {
        let tax_id = name_to_id
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("No such name: {name}"))?;
        tax_ids.push(*tax_id);
    }

    Ok(tax_ids)
}

/// IDs to Nodes
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// let ids = vec![12340, 12347];
/// let taxa = nwr::get_taxon(&conn, &ids).unwrap();
///
/// assert_eq!(taxa.get(0).unwrap().tax_id, 12340);
/// assert_eq!(taxa.get(0).unwrap().parent_tax_id, 12333);
/// assert_eq!(taxa.get(0).unwrap().rank, "species");
/// assert_eq!(taxa.get(0).unwrap().division, "Phages");
/// assert_eq!(taxa.get(1).unwrap().tax_id, 12347);
/// ```
pub fn get_taxon(
    conn: &rusqlite::Connection,
    ids: &[i64],
) -> anyhow::Result<Vec<Taxon>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut taxa_map: HashMap<i64, Taxon> = HashMap::new();

    // Deduplicate ids before querying so that the same tax_id is never fetched
    // twice (which would push duplicate name entries across chunks). The
    // original `ids` order is preserved when building the output vector below.
    let unique_ids: Vec<i64> = {
        let mut seen = std::collections::HashSet::new();
        ids.iter().filter(|id| seen.insert(**id)).copied().collect()
    };

    for chunk in unique_ids.chunks(CHUNK_SIZE) {
        let placeholders = (0..chunk.len()).map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "
            SELECT
                node.tax_id,
                node.parent_tax_id,
                node.rank,
                division.division,
                name.name_class,
                name.name,
                node.comment
            FROM node
                INNER JOIN name ON node.tax_id = name.tax_id
                INNER JOIN division ON node.division_id = division.id
            WHERE node.tax_id IN ({placeholders})
            "
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(chunk.iter()))?;

        while let Some(row) = rows.next()? {
            let tax_id: i64 = row.get(0)?;
            let name_class: String = row.get(4)?;
            let name: String = row.get(5)?;

            if let std::collections::hash_map::Entry::Vacant(e) = taxa_map.entry(tax_id)
            {
                let mut taxon = Taxon {
                    tax_id,
                    parent_tax_id: row.get(1)?,
                    rank: row.get(2)?,
                    division: row.get(3)?,
                    ..Default::default()
                };
                let comments: String = row.get(6)?;
                if !comments.is_empty() {
                    taxon.comments = Some(comments);
                }
                e.insert(taxon);
            }

            if let Some(taxon) = taxa_map.get_mut(&tax_id) {
                taxon.names.entry(name_class).or_default().push(name);
            }
        }
    }

    // When the caller passes duplicate IDs, we must keep the map intact so
    // every occurrence can be resolved. In the common (no-duplicates) case we
    // drain the map via `remove` to avoid cloning each `Taxon` (which owns a
    // `HashMap` of names) — a significant win for large clade queries.
    let has_duplicates = ids.len() != unique_ids.len();
    let mut taxa = Vec::with_capacity(ids.len());
    for id in ids {
        let taxon = if has_duplicates {
            taxa_map
                .get(id)
                .ok_or_else(|| anyhow::anyhow!("No such ID: {id}"))?
                .clone()
        } else {
            taxa_map
                .remove(id)
                .ok_or_else(|| anyhow::anyhow!("No such ID: {id}"))?
        };
        taxa.push(taxon);
    }

    Ok(taxa)
}

/// Retrieve the ancestor
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// let ancestor = nwr::get_ancestor(&conn, 12340).unwrap();
///
/// assert_eq!(ancestor.tax_id, 12333);
/// ```
pub fn get_ancestor(conn: &rusqlite::Connection, id: i64) -> anyhow::Result<Taxon> {
    // The canonical root (tax_id 1) is its own parent, so it has no ancestor.
    if id == 1 {
        anyhow::bail!("Root (tax_id 1) has no ancestor");
    }

    let mut stmt = conn.prepare(
        "
        SELECT parent_tax_id
        FROM node
        WHERE tax_id=?
        ",
    )?;
    let parent_id = stmt.query_row([id], |row| row.get(0))?;

    // Only the canonical root (tax_id 1) may be self-referential. Any other
    // node that is its own parent indicates corrupt data; bail out instead of
    // returning the node itself as its own ancestor.
    if parent_id == id {
        anyhow::bail!("Taxon {id} is its own parent (not root)");
    }

    let ancestor = get_taxon(conn, &[parent_id])?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No ancestor found for parent ID {parent_id}"))?;

    Ok(ancestor)
}

/// Maximum recursion depth for taxonomy tree traversals.
///
/// NCBI taxonomy depths are well below this limit; the bound acts as a guard
/// against corrupt cyclic data.
const MAX_TAXONOMY_DEPTH: i64 = 1000;

/// All Nodes to the root (with ID 1)
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// let lineage = nwr::get_lineage(&conn, 12340).unwrap();
///
/// assert_eq!(lineage.get(0).unwrap().tax_id, 1);
/// assert_eq!(lineage.last().unwrap().tax_id, 12340);
/// assert_eq!(lineage.len(), 4);
/// ```
pub fn get_lineage(conn: &rusqlite::Connection, id: i64) -> anyhow::Result<Vec<Taxon>> {
    // Walk to the root in a single recursive CTE instead of issuing one query
    // per lineage level. The CTE returns rows from the starting taxon up to
    // (and including) the canonical root.
    let mut stmt = conn.prepare(
        "
        WITH RECURSIVE ancestors(tax_id, parent_tax_id, level) AS (
            SELECT tax_id, parent_tax_id, 0
            FROM node
            WHERE tax_id = ?1
            UNION ALL
            SELECT n.tax_id, n.parent_tax_id, a.level + 1
            FROM node n
            JOIN ancestors a ON n.tax_id = a.parent_tax_id
            WHERE a.tax_id != 1
              AND n.tax_id != a.tax_id
              AND a.level < ?2
        )
        SELECT tax_id, parent_tax_id
        FROM ancestors
        ORDER BY level
        ",
    )?;

    let mut rows = stmt.query(rusqlite::params![id, MAX_TAXONOMY_DEPTH])?;
    let mut ids = Vec::new();
    let mut seen = std::collections::HashSet::new();

    while let Some(row) = rows.next()? {
        let tax_id: i64 = row.get(0)?;
        let parent_id: i64 = row.get(1)?;

        // Only the canonical root may be self-referential.
        if tax_id != 1 && tax_id == parent_id {
            anyhow::bail!("Taxon {tax_id} is its own parent (not root)");
        }
        if !seen.insert(tax_id) {
            anyhow::bail!("Taxonomy cycle detected involving tax_id {tax_id}");
        }

        ids.push(tax_id);
    }

    if ids.last() != Some(&1) {
        anyhow::bail!("Lineage for tax_id {id} does not reach the root");
    }

    ids.reverse();
    let lineage = get_taxon(conn, &ids)?;

    Ok(lineage)
}

/// All direct descendents of the Node, not a recursive fetchall
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// // Synechococcus phage S
/// let descendents = nwr::get_descendent(&conn, 375032).unwrap();
///
/// assert_eq!(descendents.get(0).unwrap().tax_id, 375033);
/// assert_eq!(descendents.get(0).unwrap().rank, "no rank");
/// assert_eq!(descendents.len(), 34);
/// ```
pub fn get_descendent(
    conn: &rusqlite::Connection,
    id: i64,
) -> anyhow::Result<Vec<Taxon>> {
    let mut ids: Vec<i64> = vec![];

    let mut stmt = conn.prepare(
        "
        SELECT tax_id
        FROM node
        WHERE parent_tax_id=?
        ",
    )?;

    let mut rows = stmt.query([id])?;
    while let Some(row) = rows.next()? {
        let child_id: i64 = row.get(0)?;
        // Skip self-loop only for the canonical root (tax_id 1), which is
        // its own parent by definition. Any other self-loop is corrupt data.
        if child_id == id {
            if id == 1 {
                continue;
            }
            anyhow::bail!("Taxonomy cycle detected: tax_id {id} is its own child");
        }
        ids.push(child_id);
    }

    let nodes = get_taxon(conn, &ids)?;
    Ok(nodes)
}

/// All direct or indirect descendents of the Node.
/// The ID given as argument is included in the results.
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// // Synechococcus phage S
/// let descendents = nwr::get_all_descendent(&conn, 375032).unwrap();
///
/// assert_eq!(*descendents.get(0).unwrap(), 375032);
/// assert_eq!(descendents.len(), 35);
/// ```
pub fn get_all_descendent(
    conn: &rusqlite::Connection,
    id: i64,
) -> anyhow::Result<Vec<i64>> {
    // Fetch the entire subtree in a single recursive CTE instead of issuing
    // one query per node. The CTE starts with the requested taxon and follows
    // parent->child edges, ignoring self-loops (only the root is its own
    // parent by definition).
    let mut stmt = conn.prepare(
        "
        WITH RECURSIVE descendants(tax_id, level) AS (
            SELECT tax_id, 0
            FROM node
            WHERE tax_id = ?1
            UNION ALL
            SELECT n.tax_id, d.level + 1
            FROM node n
            JOIN descendants d ON n.parent_tax_id = d.tax_id
            WHERE n.tax_id != d.tax_id
              AND d.level < ?2
        )
        SELECT tax_id
        FROM descendants
        ORDER BY level
        ",
    )?;

    let mut rows = stmt.query(rusqlite::params![id, MAX_TAXONOMY_DEPTH])?;
    let mut ids = Vec::new();
    let mut seen = std::collections::HashSet::new();

    while let Some(row) = rows.next()? {
        let tax_id: i64 = row.get(0)?;
        if !seen.insert(tax_id) {
            anyhow::bail!("Taxonomy cycle detected involving tax_id {tax_id}");
        }
        ids.push(tax_id);
    }

    Ok(ids)
}

/// Convert terms to Taxonomy IDs
/// Accepted forms: ID; "scientific name"; `scientific_name`
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// let id = nwr::term_to_tax_id(&conn, "10239").unwrap();
/// assert_eq!(id, 10239);
///
/// let id = nwr::term_to_tax_id(&conn, "Viruses").unwrap();
/// assert_eq!(id, 10239);
///
/// let id = nwr::term_to_tax_id(&conn, "Lactobacillus phage mv4").unwrap();
/// assert_eq!(id, 12392);
///
/// let id = nwr::term_to_tax_id(&conn, "Lactobacillus_phage_mv4").unwrap();
/// assert_eq!(id, 12392);
/// ```
pub fn term_to_tax_id(conn: &rusqlite::Connection, term: &str) -> anyhow::Result<i64> {
    let term = term.trim().replace('_', " ");

    let id: i64 = if let Ok(n) = term.parse::<i64>() {
        n
    } else {
        match get_tax_id(conn, std::slice::from_ref(&term))?
            .into_iter()
            .next()
        {
            Some(id) => id,
            None => anyhow::bail!("No tax ID found for term: {term}"),
        }
    };

    Ok(id)
}

/// Batch-convert a list of terms to Taxonomy IDs.
///
/// Numeric strings are parsed directly; other strings are resolved against the
/// `name` table in a single batched query. The returned vector preserves the
/// input order.
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
///
/// let ids = nwr::terms_to_tax_ids(&conn, &["10239", "Viruses", "Lactobacillus_phage_mv4"]).unwrap();
/// assert_eq!(ids, vec![10239, 10239, 12392]);
/// ```
pub fn terms_to_tax_ids<S: AsRef<str>>(
    conn: &rusqlite::Connection,
    terms: &[S],
) -> anyhow::Result<Vec<i64>> {
    let mut ids = vec![0; terms.len()];

    let mut name_terms: Vec<(usize, String)> = Vec::new();
    for (i, term) in terms.iter().enumerate() {
        let term = term.as_ref();
        let normalized = term.trim().replace('_', " ");
        if let Ok(n) = normalized.parse::<i64>() {
            ids[i] = n;
        } else {
            name_terms.push((i, normalized));
        }
    }

    if !name_terms.is_empty() {
        let names: Vec<String> = name_terms.iter().map(|(_, n)| n.clone()).collect();
        let resolved = get_tax_id(conn, &names)
            .map_err(|e| anyhow::anyhow!("Failed to resolve one or more terms: {e}"))?;
        for ((i, _), tax_id) in name_terms.iter().zip(resolved.iter()) {
            ids[*i] = *tax_id;
        }
    }

    Ok(ids)
}

/// Find rank in lineage
///
/// Returns `(tax_id, scientific_name)` for the first node whose `rank` matches.
/// If no match is found, returns the sentinel `(0, "NA")` — callers rely on
/// this convention to represent a missing rank. The returned `&str` borrows
/// from `lineage` to avoid per-call allocations in hot loops.
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
/// let lineage = nwr::get_lineage(&conn, 12340).unwrap();
/// let (species_id, species_name) = nwr::find_rank(&lineage, "species");
/// assert_eq!(species_id, 12340);
/// assert_eq!(species_name, "Enterobacteria phage 933J");
/// ```
#[must_use]
pub fn find_rank<'a>(lineage: &'a [Taxon], rank: &str) -> (i64, &'a str) {
    for node in lineage {
        if node.rank == rank {
            return (node.tax_id, node.scientific_name().unwrap_or("NA"));
        }
    }
    (0, "NA")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taxon_display() {
        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            division: "Phages".to_string(),
            names: HashMap::from([
                (
                    "scientific name".to_string(),
                    vec!["Test Phage".to_string()],
                ),
                (
                    "synonym".to_string(),
                    vec!["Synonym1".to_string(), "Synonym2".to_string()],
                ),
            ]),
            ..Default::default()
        };
        let display = format!("{}", taxon);
        assert!(display.contains("Test Phage"));
        assert!(display.contains("12340"));
        assert!(display.contains("Phages"));
        assert!(display.contains("Synonym1"));
    }

    #[test]
    fn test_get_tax_id_not_found() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let result = get_tax_id(&conn, &["NonExistentName".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No such name"));
    }

    #[test]
    fn test_get_taxon_not_found() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let result = get_taxon(&conn, &[999999999]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No such ID"));
    }

    #[test]
    fn test_get_taxon_duplicate_ids() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        // 12340 is Enterobacteria phage 933J
        let taxa = get_taxon(&conn, &[12340, 12340]).unwrap();
        assert_eq!(taxa.len(), 2);
        assert_eq!(taxa[0].tax_id, 12340);
        assert_eq!(taxa[1].tax_id, 12340);
    }

    #[test]
    fn test_find_rank_not_found() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let lineage = get_lineage(&conn, 12340).unwrap();
        let (tax_id, sci_name) = find_rank(&lineage, "kingdom");
        assert_eq!(tax_id, 0);
        assert_eq!(sci_name, "NA");
    }

    #[test]
    fn test_taxon_display_with_genbank_name() {
        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            division: "Phages".to_string(),
            names: HashMap::from([
                (
                    "scientific name".to_string(),
                    vec!["Test Phage".to_string()],
                ),
                ("genbank common name".to_string(), vec!["Testy".to_string()]),
            ]),
            ..Default::default()
        };
        let display = format!("{}", taxon);
        assert!(display.contains("Commonly named Testy"));
    }

    #[test]
    fn test_taxon_display_with_common_names() {
        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            division: "Phages".to_string(),
            names: HashMap::from([
                (
                    "scientific name".to_string(),
                    vec!["Test Phage".to_string()],
                ),
                (
                    "common name".to_string(),
                    vec!["Common1".to_string(), "Common2".to_string()],
                ),
            ]),
            ..Default::default()
        };
        let display = format!("{}", taxon);
        assert!(display.contains("Also known as:"));
        assert!(display.contains("Common1"));
        assert!(display.contains("Common2"));
    }

    #[test]
    fn test_taxon_display_with_authority() {
        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            division: "Phages".to_string(),
            names: HashMap::from([
                (
                    "scientific name".to_string(),
                    vec!["Test Phage".to_string()],
                ),
                (
                    "authority".to_string(),
                    vec!["Smith et al. 2020".to_string()],
                ),
            ]),
            ..Default::default()
        };
        let display = format!("{}", taxon);
        assert!(display.contains("First description:"));
        assert!(display.contains("Smith et al. 2020"));
    }

    #[test]
    fn test_taxon_display_with_comments() {
        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            division: "Phages".to_string(),
            names: HashMap::from([(
                "scientific name".to_string(),
                vec!["Test Phage".to_string()],
            )]),
            comments: Some("This is a test comment".to_string()),
            ..Default::default()
        };
        let display = format!("{}", taxon);
        assert!(display.contains("Comments:"));
        assert!(display.contains("This is a test comment"));
    }

    #[test]
    fn test_taxon_display_without_scientific_name() {
        let taxon = Taxon {
            tax_id: 12340,
            rank: "species".to_string(),
            division: "Phages".to_string(),
            names: HashMap::new(),
            ..Default::default()
        };
        let display = format!("{}", taxon);
        assert!(display.contains("Unknown"));
    }

    #[test]
    fn test_term_to_tax_id_with_numeric() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        // Test with numeric string
        let id = term_to_tax_id(&conn, "10239").unwrap();
        assert_eq!(id, 10239);
    }

    #[test]
    fn test_term_to_tax_id_with_underscores() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        // Test with underscores replacing spaces
        let id = term_to_tax_id(&conn, "Lactobacillus_phage_mv4").unwrap();
        assert_eq!(id, 12392);
    }

    #[test]
    fn test_term_to_tax_id_not_found() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        let result = term_to_tax_id(&conn, "NonExistentTaxonName12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_ancestor() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        // 12340 is Enterobacteria phage 933J, parent is 12333
        let ancestor = get_ancestor(&conn, 12340).unwrap();
        assert_eq!(ancestor.tax_id, 12333);
    }

    #[test]
    fn test_get_descendent() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        // 375032 is Synechococcus phage S
        let descendents = get_descendent(&conn, 375032).unwrap();
        assert!(!descendents.is_empty());
    }

    #[test]
    fn test_get_all_descendent() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();

        // 375032 is Synechococcus phage S
        let descendents = get_all_descendent(&conn, 375032).unwrap();
        assert!(descendents.contains(&375032)); // Should include self
        assert!(descendents.len() > 1); // Should have children
    }

    #[test]
    fn test_get_all_descendent_cycle() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE node (
                tax_id        INTEGER PRIMARY KEY,
                parent_tax_id INTEGER,
                rank          VARCHAR NOT NULL,
                division_id   INTEGER NOT NULL,
                comment       TEXT
            );
            ",
        )
        .unwrap();
        // Create a two-node cycle: 2 -> 3 -> 2.
        conn.execute(
            "INSERT INTO node (tax_id, parent_tax_id, rank, division_id, comment)
             VALUES (2, 3, 'species', 1, '')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO node (tax_id, parent_tax_id, rank, division_id, comment)
             VALUES (3, 2, 'species', 1, '')",
            [],
        )
        .unwrap();

        let result = get_all_descendent(&conn, 2);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Taxonomy cycle detected"));
    }

    #[test]
    fn test_nwr_path() {
        let path = nwr_path().unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains(".nwr"));
    }

    #[test]
    fn test_get_nwr_dir_with_arg() {
        use clap::{Arg, Command};

        let cmd = Command::new("test").arg(Arg::new("dir").long("dir").num_args(1));
        let matches = cmd
            .try_get_matches_from(["test", "--dir", "tests/nwr/"])
            .unwrap();

        let path = get_nwr_dir(&matches, "dir").unwrap();
        assert_eq!(path.to_string_lossy(), "tests/nwr/");
    }

    #[test]
    fn test_get_nwr_dir_default() {
        use clap::{Arg, Command};

        let cmd = Command::new("test").arg(Arg::new("dir").long("dir").num_args(1));
        let matches = cmd.try_get_matches_from(["test"]).unwrap();

        let path = get_nwr_dir(&matches, "dir").unwrap();
        // Should return default nwr path
        assert!(path.to_string_lossy().contains(".nwr"));
    }

    #[test]
    fn test_get_lineage_self_loop_cycle() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE node (
                tax_id        INTEGER PRIMARY KEY,
                parent_tax_id INTEGER,
                rank          VARCHAR NOT NULL,
                division_id   INTEGER NOT NULL,
                comment       TEXT
            );
            ",
        )
        .unwrap();
        // Insert a non-root node that is its own parent.
        conn.execute(
            "INSERT INTO node (tax_id, parent_tax_id, rank, division_id, comment)
             VALUES (2, 2, 'species', 1, '')",
            [],
        )
        .unwrap();

        let result = get_lineage(&conn, 2);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is its own parent"));
    }

    #[test]
    fn fallback_sp_without_dot() {
        // "Cladobotryum sp" -> ["Cladobotryum sp."]
        let c = fallback_candidates("Cladobotryum sp");
        assert_eq!(c, vec!["Cladobotryum sp.".to_string()]);
    }

    #[test]
    fn fallback_sp_with_dot() {
        // "Cladobotryum sp." -> ["Cladobotryum sp"]
        let c = fallback_candidates("Cladobotryum sp.");
        assert_eq!(c, vec!["Cladobotryum sp".to_string()]);
    }

    #[test]
    fn fallback_nom_inval_no_punct() {
        // "Trichoderma carraovejensis nom inval" -> ["Trichoderma carraovejensis"]
        let c = fallback_candidates("Trichoderma carraovejensis nom inval");
        assert_eq!(c, vec!["Trichoderma carraovejensis".to_string()]);
    }

    #[test]
    fn fallback_nom_inval_with_parens() {
        // "Trichoderma carraovejensis (nom. inval.)" -> ["Trichoderma carraovejensis"]
        let c = fallback_candidates("Trichoderma carraovejensis (nom. inval.)");
        assert_eq!(c, vec!["Trichoderma carraovejensis".to_string()]);
    }

    #[test]
    fn fallback_no_match() {
        // Names without sp/nom qualifiers produce no fallback candidates
        let c = fallback_candidates("not_a_real_taxon_name");
        assert!(c.is_empty());
    }

    #[test]
    fn fallback_sp_in_middle_not_matched() {
        // "sp" in the middle should not trigger fallback
        let c = fallback_candidates("Homo sapiens");
        assert!(c.is_empty());
    }

    #[test]
    fn fallback_cf_not_stripped() {
        // cf. is an independent taxon, must NOT be stripped (would mismatch)
        // e.g. "Trichoderma cf. harzianum" (1715252) != "Trichoderma harzianum" (5544)
        let c = fallback_candidates("Trichoderma cf. harzianum");
        assert!(c.is_empty(), "cf. must not generate fallback candidates");
    }

    #[test]
    fn fallback_aff_not_stripped() {
        // aff. is an independent taxon, must NOT be stripped (would mismatch)
        let c = fallback_candidates("Trichoderma aff. harzianum");
        assert!(c.is_empty(), "aff. must not generate fallback candidates");
    }
}

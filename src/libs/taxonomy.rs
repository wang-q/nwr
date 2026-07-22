use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// Abbreviation generation for strain/species/genus names.
pub mod abbr;
/// Append taxonomic rank columns to TSV files.
pub mod append;
/// Shared helpers for taxonomy tree operations.
pub mod common;
/// Display information for taxonomy IDs or names.
pub mod info;
/// Output the lineage of a taxonomy term.
pub mod lineage;
/// List taxonomy members under ancestor terms.
pub mod member;
/// Restrict TSV lines to descendants of ancestor terms.
pub mod restrict;

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
    pub fn scientific_name(&self) -> Option<&str> {
        self.names
            .get("scientific name")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }
}

impl std::fmt::Display for Taxon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut lines = String::new();

        let sciname = self.scientific_name().unwrap_or("Unknown");
        let l1 = format!("{} - {}\n", sciname, self.rank);
        let l2 = "-".repeat(l1.chars().count() - 1);
        lines.push_str(&l1);
        lines.push_str(&l2);
        let _ = writeln!(lines, "\nNCBI Taxonomy ID: {}", self.tax_id);

        if let Some(synonyms) = self.names.get("synonym") {
            lines.push_str("Same as:\n");
            for synonym in synonyms {
                let _ = writeln!(lines, "* {}", synonym);
            }
        }

        if let Some(genbank_names) = self.names.get("genbank common name") {
            if let Some(genbank) = genbank_names.first() {
                let _ = writeln!(lines, "Commonly named {}.", genbank);
            }
        }

        if let Some(common_names) = self.names.get("common name") {
            lines.push_str("Also known as:\n");
            for name in common_names {
                let _ = writeln!(lines, "* {}", name);
            }
        }

        if let Some(authorities) = self.names.get("authority") {
            lines.push_str("First description:\n");
            for authority in authorities {
                let _ = writeln!(lines, "* {}", authority);
            }
        }

        let _ = writeln!(lines, "Part of the {}.", self.division);

        if let Some(ref comments) = self.comments {
            let _ = writeln!(lines, "\nComments: {}", comments);
        }

        write!(f, "{}", lines)
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

/// Get nwr working directory from command line args or default path
///
/// # Arguments
/// * `args` - Command line arguments from clap
/// * `arg_name` - The name of the directory argument (typically "dir")
///
/// # Returns
/// * `anyhow::Result<PathBuf>` - The resolved path
///
/// # Example
/// ```ignore
/// let nwrdir = nwr::get_nwr_dir(&args, "dir").unwrap();
/// ```
pub fn get_nwr_dir(
    args: &clap::ArgMatches,
    arg_name: &str,
) -> anyhow::Result<std::path::PathBuf> {
    if let Some(dir) = args.get_one::<String>(arg_name) {
        Ok(Path::new(dir).to_path_buf())
    } else {
        nwr_path()
    }
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
    let conn = rusqlite::Connection::open(dbfile)?;

    Ok(conn)
}

/// Names to Taxonomy IDs
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
    let mut tax_ids = vec![];

    let mut stmt = conn.prepare(
        "
        SELECT tax_id FROM name
        WHERE name_class IN ('scientific name', 'synonym', 'genbank synonym')
        AND name=?
        ORDER BY tax_id
        ",
    )?;

    for name in names {
        let mut rows = stmt.query([name])?;

        if let Some(row) = rows.next()? {
            tax_ids.push(row.get(0)?);
        } else {
            return Err(anyhow::anyhow!("No such name: {}", name));
        }
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

    // Chunk ids to stay below SQLite's bound-variable limit (999 by default,
    // 32766 on newer builds). Large clades such as Bacteria yield hundreds of
    // thousands of ids via `get_all_descendent`, which would otherwise exceed
    // the limit and fail at runtime.
    const CHUNK_SIZE: usize = 900;

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
            WHERE node.tax_id IN ({})
            ORDER BY name.name
            ",
            placeholders
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
                .ok_or_else(|| anyhow::anyhow!("No such ID: {}", id))?
                .clone()
        } else {
            taxa_map
                .remove(id)
                .ok_or_else(|| anyhow::anyhow!("No such ID: {}", id))?
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
        anyhow::bail!("Taxon {} is its own parent (not root)", id);
    }

    let ancestor = get_taxon(conn, &[parent_id])?
        .into_iter()
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("No ancestor found for parent ID {}", parent_id)
        })?;

    Ok(ancestor)
}

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
    let mut id = id;
    let mut ids = vec![id];
    let mut seen = std::collections::HashSet::new();
    seen.insert(id);

    let mut stmt = conn.prepare(
        "
        SELECT parent_tax_id
        FROM node
        WHERE tax_id=?
        ",
    )?;

    loop {
        // Reached the canonical root; no need to query its parent again.
        if id == 1 {
            break;
        }

        let parent_id = stmt.query_row([id], |row| row.get(0))?;
        ids.push(parent_id);

        // Only the canonical root (tax_id 1) may be self-referential.
        if parent_id == id {
            return Err(anyhow::anyhow!(
                "Taxonomy cycle detected: tax_id {} is its own parent",
                id
            ));
        }

        if !seen.insert(parent_id) {
            return Err(anyhow::anyhow!(
                "Taxonomy cycle detected involving tax_id {}",
                parent_id
            ));
        }

        id = parent_id;
    }

    // `ids` is already unique: the `seen` set rejects duplicates above.
    let mut lineage = get_taxon(conn, &ids)?;
    lineage.reverse();

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
        // Skip self-loop: the canonical root (tax_id 1) is its own parent.
        // For any other node, a self-loop indicates corrupt data.
        if child_id == id {
            continue;
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
    let mut ids: Vec<i64> = vec![];
    let mut temp_ids = vec![id];
    let mut seen = std::collections::HashSet::new();

    let mut stmt = conn.prepare(
        "
        SELECT tax_id
        FROM node
        WHERE parent_tax_id=?
        ",
    )?;

    while let Some(id) = temp_ids.pop() {
        if !seen.insert(id) {
            return Err(anyhow::anyhow!(
                "Taxonomy cycle detected involving tax_id {}",
                id
            ));
        }
        ids.push(id);

        let mut rows = stmt.query([id])?;
        while let Some(row) = rows.next()? {
            let child_id: i64 = row.get(0)?;
            // Skip self-loop: the canonical root (tax_id 1) is its own parent.
            if child_id == id {
                continue;
            }
            temp_ids.push(child_id);
        }
    }

    // `ids` is already unique: the `seen` set rejects duplicates above.
    Ok(ids)
}

/// Convert terms to Taxonomy IDs
/// Accepted forms: ID; "scientific name"; scientific_name
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

    let id: i64 = match term.parse::<i64>() {
        Ok(n) => n,
        Err(_) => {
            let ids = get_tax_id(conn, &[term])?;
            ids.into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No tax ID found for term"))?
        }
    };

    Ok(id)
}

/// Find rank in lineage
///
/// Returns `(tax_id, scientific_name)` for the first node whose `rank` matches.
/// If no match is found, returns the sentinel `(0, "NA")` — callers rely on
/// this convention to represent a missing rank.
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
/// let lineage = nwr::get_lineage(&conn, 12340).unwrap();
/// let (species_id, species_name) = nwr::find_rank(&lineage, "species");
/// assert_eq!(species_id, 12340);
/// assert_eq!(species_name, "Enterobacteria phage 933J");
/// ```
pub fn find_rank(lineage: &[Taxon], rank: &str) -> (i64, String) {
    let mut tax_id: i64 = 0;
    let mut sci_name = "NA".to_string();

    for node in lineage.iter() {
        if node.rank == rank {
            sci_name = node.scientific_name().unwrap_or("NA").to_string();
            tax_id = node.tax_id;
            break;
        }
    }

    (tax_id, sci_name)
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
}

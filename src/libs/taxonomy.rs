use itertools::Itertools;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Taxon {
    pub tax_id: i64,
    pub parent_tax_id: i64,
    pub rank: String,
    pub division: String,
    pub names: HashMap<String, Vec<String>>, // many synonym or common names
    pub comments: Option<String>,
}

impl std::fmt::Display for Taxon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut lines = String::new();

        let sciname = self
            .names
            .get("scientific name")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let l1 = format!("{} - {}\n", sciname, self.rank);
        let l2 = "-".repeat(l1.len() - 1);
        lines.push_str(&l1);
        lines.push_str(&l2);
        lines.push_str(&format!("\nNCBI Taxonomy ID: {}\n", self.tax_id));

        if let Some(synonyms) = self.names.get("synonym") {
            lines.push_str("Same as:\n");
            for synonym in synonyms {
                lines.push_str(&format!("* {}\n", synonym));
            }
        }

        if let Some(genbank_names) = self.names.get("genbank common name") {
            if let Some(genbank) = genbank_names.first() {
                lines.push_str(&format!("Commonly named {}.\n", genbank));
            }
        }

        if let Some(common_names) = self.names.get("common name") {
            lines.push_str("Also known as:\n");
            for name in common_names {
                lines.push_str(&format!("* {}\n", name));
            }
        }

        if let Some(authorities) = self.names.get("authority") {
            lines.push_str("First description:\n");
            for authority in authorities {
                lines.push_str(&format!("* {}\n", authority));
            }
        }

        lines.push_str(&format!("Part of the {}.\n", self.division));

        if let Some(ref comments) = self.comments {
            lines.push_str(&format!("\nComments: {}", comments));
        }

        writeln!(f, "{}\n", lines)
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
    if args.contains_id(arg_name) {
        Ok(Path::new(args.get_one::<String>(arg_name).unwrap()).to_path_buf())
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
/// let tax_ids = nwr::get_tax_id(&conn, names).unwrap();
///
/// assert_eq!(tax_ids, vec![12340, 12347]);
/// ```
pub fn get_tax_id(
    conn: &rusqlite::Connection,
    names: Vec<String>,
) -> anyhow::Result<Vec<i64>> {
    let mut tax_ids = vec![];

    let mut stmt = conn.prepare(
        "
        SELECT tax_id FROM name
        WHERE 1=1
        AND name_class IN ('scientific name', 'synonym', 'genbank synonym')
        AND name=?
        ",
    )?;

    for name in names.iter() {
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
/// let taxa = nwr::get_taxon(&conn, ids).unwrap();
///
/// assert_eq!(taxa.get(0).unwrap().tax_id, 12340);
/// assert_eq!(taxa.get(0).unwrap().parent_tax_id, 12333);
/// assert_eq!(taxa.get(0).unwrap().rank, "species");
/// assert_eq!(taxa.get(0).unwrap().division, "Phages");
/// assert_eq!(taxa.get(1).unwrap().tax_id, 12347);
/// ```
pub fn get_taxon(
    conn: &rusqlite::Connection,
    ids: Vec<i64>,
) -> anyhow::Result<Vec<Taxon>> {
    let mut taxa = vec![];

    let mut stmt = conn.prepare(
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
        WHERE node.tax_id=?
        ",
    )?;

    for id in ids.iter() {
        let mut rows = stmt.query([id])?;

        let mut taxon: Taxon = Default::default();
        if let Some(row) = rows.next()? {
            taxon.tax_id = row.get(0)?;
            taxon.parent_tax_id = row.get(1)?;
            taxon.rank = row.get(2)?;
            taxon.division = row.get(3)?;

            let comments: String = row.get(6)?;
            if !comments.is_empty() {
                taxon.comments = Some(comments);
            }

            let name_class: String = row.get(4)?;
            let name: String = row.get(5)?;
            taxon.names.entry(name_class).or_insert_with(|| vec![name]);
        } else {
            return Err(anyhow::anyhow!("No such ID: {}", id));
        }

        while let Some(row) = rows.next()? {
            let name_class: String = row.get(4)?;
            let name: String = row.get(5)?;
            taxon
                .names
                .entry(name_class)
                .and_modify(|n| n.push(name.clone()))
                .or_insert_with(|| vec![name]);
        }

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
    let mut stmt = conn.prepare(
        "
        SELECT parent_tax_id
        FROM node
        WHERE tax_id=?
        ",
    )?;
    let parent_id = stmt.query_row([id], |row| row.get(0))?;

    let ancestor = get_taxon(conn, vec![parent_id])?.pop().unwrap();

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

    let mut stmt = conn.prepare(
        "
        SELECT parent_tax_id
        FROM node
        WHERE tax_id=?
        ",
    )?;

    loop {
        let parent_id = stmt.query_row([id], |row| row.get(0))?;
        ids.push(parent_id);

        // the root or one of the roots
        if id == 1 || parent_id == id {
            break;
        }

        id = parent_id;
    }

    let ids: Vec<_> = ids.into_iter().unique().collect();
    let mut lineage = get_taxon(conn, ids)?;
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
        ids.push(row.get(0)?);
    }

    let nodes = get_taxon(conn, ids)?;
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

    let mut stmt = conn.prepare(
        "
        SELECT tax_id
        FROM node
        WHERE parent_tax_id=?
        ",
    )?;

    while let Some(id) = temp_ids.pop() {
        ids.push(id);

        let mut rows = stmt.query([id])?;
        while let Some(row) = rows.next()? {
            temp_ids.push(row.get(0)?);
        }
    }

    let ids: Vec<_> = ids.into_iter().unique().collect();
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
            let ids = get_tax_id(conn, vec![term])?;
            ids.into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No tax ID found for term"))?
        }
    };

    Ok(id)
}

/// Find rank in lineage
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
/// let lineage = nwr::get_lineage(&conn, 12340).unwrap();
/// let (species_id, species_name) = nwr::find_rank(&lineage, "species".to_string());
/// assert_eq!(species_id, 12340);
/// assert_eq!(species_name, "Enterobacteria phage 933J");
/// ```
pub fn find_rank(lineage: &[Taxon], rank: String) -> (i64, String) {
    let mut tax_id: i64 = 0;
    let mut sci_name = "NA".to_string();

    for node in lineage.iter() {
        if node.rank == rank {
            sci_name = node
                .names
                .get("scientific name")
                .and_then(|v| v.first())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "NA".to_string());
            tax_id = node.tax_id;
            break;
        }
    }

    (tax_id, sci_name)
}

/// Helper function to handle batch execution of SQL statements
///
/// ```
/// let path = std::path::PathBuf::from("tests/nwr/");
/// let conn = nwr::connect_txdb(&path).unwrap();
/// let mut stmts = vec![String::from("BEGIN;")];
/// stmts.push(String::from("SELECT 1;"));
/// let result = nwr::batch_exec(&conn, &mut stmts, 1001);
/// assert!(result.is_ok());
/// ```
pub fn batch_exec(
    conn: &rusqlite::Connection,
    stmts: &mut Vec<String>,
    i: usize,
) -> anyhow::Result<()> {
    if i > 1 && i.is_multiple_of(1000) {
        stmts.push(String::from("COMMIT;"));
        let stmt = &stmts.join("\n");
        conn.execute_batch(stmt)?;
        stmts.clear();
        stmts.push(String::from("BEGIN;"));
    }
    if i == usize::MAX {
        stmts.push(String::from("COMMIT;"));
        let stmt = &stmts.join("\n");
        conn.execute_batch(stmt)?;
        println!("\n    Finished");
    }
    if i > 1 && i.is_multiple_of(10000) {
        print!(".");
        std::io::stdout().flush()?; // Ensure the dot is printed immediately
    }
    Ok(())
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
                ("scientific name".to_string(), vec!["Test Phage".to_string()]),
                ("synonym".to_string(), vec!["Synonym1".to_string(), "Synonym2".to_string()]),
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
        let result = get_tax_id(&conn, vec!["NonExistentName".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No such name"));
    }

    #[test]
    fn test_get_taxon_not_found() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let result = get_taxon(&conn, vec![999999999]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No such ID"));
    }

    #[test]
    fn test_find_rank_not_found() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let lineage = get_lineage(&conn, 12340).unwrap();
        let (tax_id, sci_name) = find_rank(&lineage, "kingdom".to_string());
        assert_eq!(tax_id, 0);
        assert_eq!(sci_name, "NA");
    }

    #[test]
    fn test_batch_exec_with_commit() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let mut stmts = vec![String::from("BEGIN;")];
        
        // Test at 1000 boundary - should trigger COMMIT
        let result = batch_exec(&conn, &mut stmts, 1001);
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_exec_with_progress() {
        let path = std::path::PathBuf::from("tests/nwr/");
        let conn = connect_txdb(&path).unwrap();
        let mut stmts = vec![String::from("BEGIN;")];
        
        // Test at 10000 boundary - should print progress dot
        let result = batch_exec(&conn, &mut stmts, 10001);
        assert!(result.is_ok());
    }
}

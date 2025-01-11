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

        let sciname = &self.names.get("scientific name").unwrap()[0];
        let l1 = format!("{} - {}\n", sciname, self.rank);
        let l2 = "-".repeat(l1.len() - 1);
        lines.push_str(&l1);
        lines.push_str(&l2);
        lines.push_str(&format!("\nNCBI Taxonomy ID: {}\n", self.tax_id));

        if self.names.contains_key("synonym") {
            lines.push_str("Same as:\n");
            for synonym in self.names.get("synonym").unwrap() {
                lines.push_str(&format!("* {}\n", synonym));
            }
        }

        if self.names.contains_key("genbank common name") {
            let genbank = &self.names.get("genbank common name").unwrap()[0];
            lines.push_str(&format!("Commonly named {}.\n", genbank));
        }

        if self.names.contains_key("common name") {
            lines.push_str("Also known as:\n");
            for name in self.names.get("common name").unwrap() {
                lines.push_str(&format!("* {}\n", name));
            }
        }

        if self.names.contains_key("authority") {
            lines.push_str("First description:\n");
            for authority in self.names.get("authority").unwrap() {
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
/// let path = nwr::nwr_path();
///
/// assert!(std::path::Path::new(&path).exists());
/// ```
pub fn nwr_path() -> std::path::PathBuf {
    let path = dirs::home_dir().unwrap().join(".nwr/");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    path
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

        if let Some(row) = rows.next().unwrap() {
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
        // Here, row.get has no reason to return an error
        // so row.get_unwrap should be safe
        if let Some(row) = rows.next().unwrap() {
            taxon.tax_id = row.get(0)?;
            taxon.parent_tax_id = row.get(1)?;
            taxon.rank = row.get(2)?;
            taxon.division = row.get(3)?;

            let comments: String = row.get(6)?;
            if !comments.is_empty() {
                taxon.comments = Some(comments);
            }

            taxon
                .names
                .entry(row.get(4)?)
                .or_insert_with(|| vec![row.get(5).unwrap()]);
        } else {
            return Err(anyhow::anyhow!("No such ID: {}", id));
        }

        while let Some(row) = rows.next().unwrap() {
            taxon
                .names
                .entry(row.get(4).unwrap())
                .and_modify(|n| n.push(row.get(5).unwrap()))
                .or_insert_with(|| vec![row.get(5).unwrap()]);
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
    while let Some(row) = rows.next().unwrap() {
        ids.push(row.get(0).unwrap());
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
        while let Some(row) = rows.next().unwrap() {
            temp_ids.push(row.get(0).unwrap());
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
        Err(_) => get_tax_id(conn, vec![term])?.pop().unwrap(),
    };

    Ok(id)
}

pub fn find_rank(lineage: &[Taxon], rank: String) -> (i64, String) {
    let mut tax_id: i64 = 0;
    let mut sci_name = "NA".to_string();

    for node in lineage.iter() {
        if node.rank == rank {
            sci_name = node.names.get("scientific name").unwrap()[0].to_string();
            tax_id = node.tax_id;
            break;
        }
    }

    (tax_id, sci_name)
}

/// Helper function to handle batch execution of SQL statements
pub fn batch_exec(
    conn: &rusqlite::Connection,
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
    if i == usize::MAX {
        stmts.push(String::from("COMMIT;"));
        let stmt = &stmts.join("\n");
        conn.execute_batch(stmt)?;
        println!("\n    Finished");
    }
    if i > 1 && i % 10000 == 0 {
        print!(".");
        std::io::stdout().flush()?; // Ensure the dot is printed immediately
    }
    Ok(())
}

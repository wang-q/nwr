use crate::Node;
use itertools::Itertools;
use std::path::PathBuf;

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
pub fn connect_txdb(dir: &PathBuf) -> Result<rusqlite::Connection, Box<dyn std::error::Error>> {
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
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
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
        let mut rows = stmt.query(&[name])?;

        if let Some(row) = rows.next().unwrap() {
            tax_ids.push(row.get(0)?);
        } else {
            return Err(From::from(format!("No such name: {}", name)));
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
/// let nodes = nwr::get_node(&conn, ids).unwrap();
///
/// assert_eq!(nodes.get(0).unwrap().tax_id, 12340);
/// assert_eq!(nodes.get(0).unwrap().parent_tax_id, 12333);
/// assert_eq!(nodes.get(0).unwrap().rank, "species");
/// assert_eq!(nodes.get(0).unwrap().division, "Phages");
/// assert_eq!(nodes.get(1).unwrap().tax_id, 12347);
/// ```
pub fn get_node(
    conn: &rusqlite::Connection,
    ids: Vec<i64>,
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let mut nodes = vec![];

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
        let mut rows = stmt.query(&[id])?;

        let mut node: Node = Default::default();
        // Here, row.get has no reason to return an error
        // so row.get_unwrap should be safe
        if let Some(row) = rows.next().unwrap() {
            node.tax_id = row.get(0)?;
            node.parent_tax_id = row.get(1)?;
            node.rank = row.get(2)?;
            node.division = row.get(3)?;

            let comments: String = row.get(6)?;
            if !comments.is_empty() {
                node.comments = Some(comments);
            }

            node.names
                .entry(row.get(4)?)
                .or_insert_with(|| vec![row.get(5).unwrap()]);
        } else {
            return Err(From::from(format!("No such ID: {}", id)));
        }

        while let Some(row) = rows.next().unwrap() {
            node.names
                .entry(row.get(4).unwrap())
                .and_modify(|n| n.push(row.get(5).unwrap()))
                .or_insert_with(|| vec![row.get(5).unwrap()]);
        }

        nodes.push(node);
    }

    Ok(nodes)
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
pub fn get_ancestor(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Node, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT parent_tax_id
        FROM node
        WHERE tax_id=?
        ",
    )?;
    let parent_id = stmt.query_row([id], |row| row.get(0))?;

    let ancestor = get_node(conn, vec![parent_id])?.pop().unwrap();

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
pub fn get_lineage(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
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
    let mut lineage = get_node(conn, ids)?;
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
) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
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

    let nodes = get_node(conn, ids)?;
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
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
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
/// let id = nwr::term_to_tax_id(&conn, "10239".to_string()).unwrap();
/// assert_eq!(id, 10239);
///
/// let id = nwr::term_to_tax_id(&conn, "Viruses".to_string()).unwrap();
/// assert_eq!(id, 10239);
///
/// let id = nwr::term_to_tax_id(&conn, "Lactobacillus phage mv4".to_string()).unwrap();
/// assert_eq!(id, 12392);
///
/// let id = nwr::term_to_tax_id(&conn, "Lactobacillus_phage_mv4".to_string()).unwrap();
/// assert_eq!(id, 12392);
///
/// ```
pub fn term_to_tax_id(
    conn: &rusqlite::Connection,
    term: String,
) -> Result<i64, Box<dyn std::error::Error>> {
    let term = term.trim().replace("_", " ");

    let id: i64 = match term.parse::<i64>() {
        Ok(n) => n,
        Err(_) => {
            let name_id = get_tax_id(conn, vec![term]).unwrap().pop().unwrap();
            name_id
        }
    };

    Ok(id)
}

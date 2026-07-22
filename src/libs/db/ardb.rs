use lazy_static::lazy_static;
use log::{debug, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

lazy_static! {
    /// Organism names matching this regex are considered incompetent and skipped.
    static ref RE_INCOMPETENT: Regex =
        Regex::new(r"(?xi)\b(uncultured|unidentified|bacterium|archaeon|metagenome)\b")
            .unwrap();

    /// Organism names matching this regex are considered viral and skipped.
    static ref RE_VIRUS: Regex = Regex::new(r"(?xi)(virus|phage)\b").unwrap();
}

/// DDL for the assembly report SQLite database.
static DDL_AR: &str = r"
DROP TABLE IF EXISTS ar;

CREATE TABLE ar (
    tax_id             INTEGER,
    organism_name      VARCHAR (200),
    infraspecific_name VARCHAR (200),
    bioproject         VARCHAR (50),
    biosample          VARCHAR (50),
    assembly_accession VARCHAR (50),
    refseq_category    VARCHAR (50),
    assembly_level     VARCHAR (50),
    genome_rep         VARCHAR (50),
    seq_rel_date       DATE,
    asm_name           VARCHAR (200),
    gbrs_paired_asm    VARCHAR (200),
    ftp_path           VARCHAR (200),
    species            VARCHAR (50),
    species_id         INTEGER,
    genus              VARCHAR (50),
    genus_id           INTEGER,
    family             VARCHAR (50),
    family_id          INTEGER
);

";

/// Column indices in NCBI `assembly_summary_refseq.txt` / `assembly_summary_genbank.txt`.
const COL_ASSEMBLY_ACCESSION: usize = 0;
const COL_BIOPROJECT: usize = 1;
const COL_BIOSAMPLE: usize = 2;
const COL_REFSEQ_CATEGORY: usize = 4;
const COL_TAX_ID: usize = 5;
const COL_ORGANISM_NAME: usize = 7;
const COL_INFRASPECIFIC_NAME: usize = 8;
const COL_ASSEMBLY_LEVEL: usize = 11;
const COL_GENOME_REP: usize = 13;
const COL_SEQ_REL_DATE: usize = 14;
const COL_ASM_NAME: usize = 15;
const COL_GBRS_PAIRED_ASM: usize = 17;
const COL_FTP_PATH: usize = 19;

/// Build the assembly database from an NCBI assembly summary file.
///
/// `tx_conn` is a connection to the taxonomy database used to resolve lineages.
/// Set `is_genbank` to `true` to load `assembly_summary_genbank.txt`, otherwise
/// `assembly_summary_refseq.txt` is loaded.
pub fn run(
    nwrdir: &std::path::Path,
    is_genbank: bool,
    tx_conn: &rusqlite::Connection,
) -> anyhow::Result<()> {
    let file = if is_genbank {
        nwrdir.join("ar_genbank.sqlite")
    } else {
        nwrdir.join("ar_refseq.sqlite")
    };
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
    conn.execute_batch(DDL_AR)?;

    info!("==> Loading...");
    let summary_file = if is_genbank {
        File::open(nwrdir.join("assembly_summary_genbank.txt"))?
    } else {
        File::open(nwrdir.join("assembly_summary_refseq.txt"))?
    };
    let rdr = BufReader::new(summary_file);

    let mut stmt = conn.prepare(
        "INSERT INTO ar(
            tax_id, organism_name, infraspecific_name, bioproject, biosample, assembly_accession, refseq_category,
            assembly_level, genome_rep, seq_rel_date, asm_name, gbrs_paired_asm, ftp_path,
            species, species_id, genus, genus_id, family, family_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)"
    )?;

    // Intentionally use explicit SQL BEGIN/COMMIT rather than rusqlite::Transaction.
    conn.execute_batch("BEGIN;")?;
    let mut lineage_cache: HashMap<i64, Vec<crate::Taxon>> = HashMap::new();
    let mut inserted: usize = 0;
    for (i, line) in rdr.lines().enumerate() {
        let line_num = i + 1;
        let line = line?;
        if line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() <= COL_FTP_PATH {
            debug!(
                "Skipping line {}: insufficient fields ({} <= {})",
                line_num,
                fields.len(),
                COL_FTP_PATH
            );
            continue;
        }

        let tax_id = fields
            .get(COL_TAX_ID)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Missing tax_id field at line {}", line_num))?
            .parse::<i64>()
            .map_err(|e| {
                anyhow::anyhow!("Invalid tax_id at line {}: {}", line_num, e)
            })?;
        let organism_name = fields.get(COL_ORGANISM_NAME).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing organism_name field at line {}", line_num)
        })?;
        let infraspecific_name =
            fields.get(COL_INFRASPECIFIC_NAME).copied().ok_or_else(|| {
                anyhow::anyhow!("Missing infraspecific_name field at line {}", line_num)
            })?;
        let bioproject = fields.get(COL_BIOPROJECT).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing bioproject field at line {}", line_num)
        })?;
        let biosample = fields.get(COL_BIOSAMPLE).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing biosample field at line {}", line_num)
        })?;
        let assembly_accession =
            fields.get(COL_ASSEMBLY_ACCESSION).copied().ok_or_else(|| {
                anyhow::anyhow!("Missing assembly_accession field at line {}", line_num)
            })?;
        let refseq_category =
            fields.get(COL_REFSEQ_CATEGORY).copied().ok_or_else(|| {
                anyhow::anyhow!("Missing refseq_category field at line {}", line_num)
            })?;
        let assembly_level =
            fields.get(COL_ASSEMBLY_LEVEL).copied().ok_or_else(|| {
                anyhow::anyhow!("Missing assembly_level field at line {}", line_num)
            })?;
        let genome_rep = fields.get(COL_GENOME_REP).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing genome_rep field at line {}", line_num)
        })?;
        let seq_rel_date = fields.get(COL_SEQ_REL_DATE).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing seq_rel_date field at line {}", line_num)
        })?;
        let asm_name = fields.get(COL_ASM_NAME).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing asm_name field at line {}", line_num)
        })?;
        let gbrs_paired_asm =
            fields.get(COL_GBRS_PAIRED_ASM).copied().ok_or_else(|| {
                anyhow::anyhow!("Missing gbrs_paired_asm field at line {}", line_num)
            })?;
        let ftp_path = fields.get(COL_FTP_PATH).copied().ok_or_else(|| {
            anyhow::anyhow!("Missing ftp_path field at line {}", line_num)
        })?;

        // clean NA/na
        let infraspecific_name = if infraspecific_name.eq_ignore_ascii_case("NA") {
            ""
        } else {
            infraspecific_name
        };

        // Skip incompetent strains
        if RE_INCOMPETENT.is_match(organism_name) {
            debug!(
                "Skipping line {}: incompetent organism name '{}'",
                line_num, organism_name
            );
            continue;
        }

        // Skip viral strains
        if RE_VIRUS.is_match(organism_name) {
            debug!(
                "Skipping line {}: viral organism name '{}'",
                line_num, organism_name
            );
            continue;
        }

        // lineage (cached to avoid repeated SQL queries for shared tax_ids)
        let lineage = lineage_cache.entry(tax_id).or_insert_with(|| {
            match crate::get_lineage(tx_conn, tax_id) {
                Err(err) => {
                    warn!("Errors on get_lineage({}): {}", tax_id, err);
                    // Use a clearly-marked missing taxon so that find_rank
                    // returns (0, "NA") for species/genus/family.
                    let taxon = crate::Taxon {
                        tax_id: 0,
                        rank: "no rank".to_string(),
                        names: HashMap::from([(
                            "scientific name".to_string(),
                            vec!["NA".to_string()],
                        )]),
                        ..Default::default()
                    };
                    vec![taxon]
                }
                Ok(x) => x,
            }
        });
        let (species_id, species) = crate::find_rank(lineage, "species");
        let (genus_id, genus) = crate::find_rank(lineage, "genus");
        let (family_id, family) = crate::find_rank(lineage, "family");

        stmt.execute(rusqlite::params![
            tax_id,
            organism_name,
            infraspecific_name,
            bioproject,
            biosample,
            assembly_accession,
            refseq_category,
            assembly_level,
            genome_rep,
            seq_rel_date.replace('/', "-"),
            asm_name,
            gbrs_paired_asm,
            ftp_path,
            species,
            species_id,
            genus,
            genus_id,
            family,
            family_id,
        ])?;

        inserted += 1;
        crate::libs::io::progress_dot(inserted)?;
    }
    eprintln!();
    conn.execute_batch("COMMIT;")?;

    debug!("Creating indexes for ar");
    conn.execute("CREATE INDEX idx_ar_tax_id ON ar(tax_id);", [])?;
    conn.execute("CREATE INDEX idx_ar_species ON ar(species);", [])?;
    conn.execute("CREATE INDEX idx_ar_species_id ON ar(species_id);", [])?;
    conn.execute("CREATE INDEX idx_ar_genus ON ar(genus);", [])?;
    conn.execute("CREATE INDEX idx_ar_genus_id ON ar(genus_id);", [])?;
    conn.execute("CREATE INDEX idx_ar_family ON ar(family);", [])?;
    conn.execute("CREATE INDEX idx_ar_family_id ON ar(family_id);", [])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_incompetent_patterns() {
        assert!(RE_INCOMPETENT.is_match("uncultured"));
        assert!(RE_INCOMPETENT.is_match("UNIDENTIFIED"));
        assert!(RE_INCOMPETENT.is_match("Bacterium"));
        assert!(RE_INCOMPETENT.is_match("Archaeon"));
        assert!(RE_INCOMPETENT.is_match("Metagenome"));
    }

    #[test]
    fn test_re_virus_patterns() {
        assert!(RE_VIRUS.is_match("virus"));
        assert!(RE_VIRUS.is_match("VIRUS"));
        assert!(RE_VIRUS.is_match("phage"));
        assert!(RE_VIRUS.is_match("PHAGE"));
    }
}

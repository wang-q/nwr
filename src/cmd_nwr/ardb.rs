use super::args;
use clap::{Arg, ArgAction, ArgMatches, Command};
use log::{debug, info, warn};
use regex::Regex;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::LazyLock;

/// Organism names matching this regex are considered incompetent and skipped.
static RE_INCOMPETENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?xi)\b(uncultured|unidentified|bacterium|archaeon|metagenome)\b")
        .unwrap()
});

/// Organism names matching this regex are considered viral and skipped.
static RE_VIRUS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?xi)(virus|phage)\b").unwrap());

/// DDL for the assembly report `SQLite` database.
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

/// Create clap subcommand arguments.
#[must_use]
pub fn make_subcommand() -> Command {
    Command::new("ardb")
        .about("Initializes the assembly database")
        .after_help(include_str!("../../docs/help/ardb.md"))
        .arg(args::dir_arg())
        .arg(
            Arg::new("genbank")
                .long("genbank")
                .action(ArgAction::SetTrue)
                .help("Create the GenBank assembly database"),
        )
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // Ignore re-initialization errors so that tests or other callers that
    // already set up a logger do not fail here.
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    );

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;
    let is_genbank = args.get_flag("genbank");
    let tx_conn = nwr::connect_txdb(&nwrdir)?;

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
    nwr::libs::db::apply_import_pragmas(&conn)?;

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
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
    )?;

    // Intentionally use explicit SQL BEGIN/COMMIT rather than rusqlite::Transaction.
    conn.execute_batch("BEGIN;")?;
    let mut lineage_cache: HashMap<i64, Vec<nwr::Taxon>> = HashMap::new();
    let mut inserted: usize = 0;
    for (i, line) in rdr.lines().enumerate() {
        let line_num = i + 1;
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        if line.trim().is_empty() {
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

        // Field accesses rely on the `fields.len() <= COL_FTP_PATH` skip above,
        // which guarantees every COL_* index is in bounds.
        let tax_id = fields[COL_TAX_ID]
            .parse::<i64>()
            .map_err(|e| anyhow::anyhow!("Invalid tax_id at line {line_num}: {e}"))?;
        let organism_name = fields[COL_ORGANISM_NAME];
        let infraspecific_name = fields[COL_INFRASPECIFIC_NAME];
        let bioproject = fields[COL_BIOPROJECT];
        let biosample = fields[COL_BIOSAMPLE];
        let assembly_accession = fields[COL_ASSEMBLY_ACCESSION];
        let refseq_category = fields[COL_REFSEQ_CATEGORY];
        let assembly_level = fields[COL_ASSEMBLY_LEVEL];
        let genome_rep = fields[COL_GENOME_REP];
        let seq_rel_date = fields[COL_SEQ_REL_DATE];
        let asm_name = fields[COL_ASM_NAME];
        let gbrs_paired_asm = fields[COL_GBRS_PAIRED_ASM];
        let ftp_path = fields[COL_FTP_PATH];

        // clean NA/na
        let infraspecific_name = if infraspecific_name.eq_ignore_ascii_case("NA") {
            ""
        } else {
            infraspecific_name
        };

        // Skip incompetent strains
        if RE_INCOMPETENT.is_match(organism_name) {
            debug!(
                "Skipping line {line_num}: incompetent organism name '{organism_name}'"
            );
            continue;
        }

        // Skip viral strains
        if RE_VIRUS.is_match(organism_name) {
            debug!("Skipping line {line_num}: viral organism name '{organism_name}'");
            continue;
        }

        // lineage (cached to avoid repeated SQL queries for shared tax_ids)
        let lineage = lineage_cache.entry(tax_id).or_insert_with(|| {
            match nwr::get_lineage(&tx_conn, tax_id) {
                Err(err) => {
                    warn!("Errors on get_lineage({tax_id}): {err}");
                    // Use a clearly-marked missing taxon so that find_rank
                    // returns (0, "NA") for species/genus/family.
                    let taxon = nwr::Taxon {
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
        let (species_id, species) = nwr::find_rank(lineage, "species");
        let (genus_id, genus) = nwr::find_rank(lineage, "genus");
        let (family_id, family) = nwr::find_rank(lineage, "family");

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
        nwr::libs::io::progress_dot(inserted)?;
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

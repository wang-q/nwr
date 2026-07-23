use super::args;
use clap::*;
use log::{debug, info, warn};
use simplelog::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use nwr::libs::db::ardb::{
    COL_ASM_NAME, COL_ASSEMBLY_ACCESSION, COL_ASSEMBLY_LEVEL, COL_BIOPROJECT,
    COL_BIOSAMPLE, COL_FTP_PATH, COL_GBRS_PAIRED_ASM, COL_GENOME_REP,
    COL_INFRASPECIFIC_NAME, COL_ORGANISM_NAME, COL_REFSEQ_CATEGORY, COL_SEQ_REL_DATE,
    COL_TAX_ID, DDL_AR, RE_INCOMPETENT, RE_VIRUS,
};

/// Create clap subcommand arguments.
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
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

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
            match nwr::get_lineage(&tx_conn, tax_id) {
                Err(err) => {
                    warn!("Errors on get_lineage({}): {}", tax_id, err);
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

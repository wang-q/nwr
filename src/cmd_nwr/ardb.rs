use clap::*;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use nwr::Node;
use regex::Regex;
use simplelog::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("ardb")
        .about("Init the assembly database")
        .after_help(
            r###"
~/.nwr/ar_refseq.sqlite
~/.nwr/ar_genbank.sqlite

* `assembly_summary_*.txt` have 23 tab-delimited columns.
* Fields with numbers are used in the database.

    0   assembly_accession  4
    1   bioproject  3
    2   biosample
    3   wgs_master
    4   refseq_category
    5   taxid AS tax_id 1
    6   species_taxid
    7   organism_name   2
    8   infraspecific_name
    9   isolate
    10  version_status
    11  assembly_level  6
    12  release_type
    13  genome_rep      7
    14  seq_rel_date    8
    15  asm_name        9
    16  submitter
    17  gbrs_paired_asm
    18  paired_asm_comp
    19  ftp_path        10
    20  excluded_from_refseq
    21  relation_to_type_material
    22  asm_not_live_date

* 6 columns appended

    11  family
    12  family_id
    13  genus
    14  genus_id
    15  species
    16  species_id

* Incompetent strains matching the following regexes in their `organism_name` were removed.

    \bCandidatus\b
    \bcandidate\b
    \buncultured\b
    \bunidentified\b
    \bbacterium\b
    \barchaeon\b
    \bmetagenome\b
    virus\b
    phage\b

* Strains with `assembly_level` of Scaffold or Contig, should have a `genome_rep` of `full`.

* The database contains one table, named `ar`

* The `SELECT` statements can be passed to SQLite as shown below:

    echo "
        SELECT
            COUNT(*)
        FROM ar
        WHERE 1=1
            AND genus IN ('Pseudomonas')
            AND assembly_level IN ('Complete Genome', 'Chromosome')
        " |
        sqlite3 -tabs ~/.nwr/ar_refseq.sqlite

* Requires SQLite version 3.34 or above.

"###,
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .takes_value(true)
                .help("Change working directory"),
        )
        .arg(
            Arg::new("genbank")
                .long("genbank")
                .help("Create the genbank ardb"),
        )
}

static DDL_AR: &str = r###"
DROP TABLE IF EXISTS ar;

CREATE TABLE IF NOT EXISTS ar (
    tax_id             INTEGER,
    organism_name      VARCHAR (255),
    bioproject         VARCHAR (50),
    assembly_accession VARCHAR (50),
    refseq_category    VARCHAR (50),
    assembly_level     VARCHAR (50),
    genome_rep         VARCHAR (50),
    seq_rel_date       DATE,
    asm_name           VARCHAR (255),
    ftp_path           VARCHAR (255),
    family             VARCHAR (50),
    family_id          INTEGER,
    genus              VARCHAR (50),
    genus_id           INTEGER,
    species            VARCHAR (50),
    species_id         INTEGER
);

"###;

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

    let nwrdir = if args.is_present("dir") {
        std::path::Path::new(args.value_of("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };
    let file = if args.is_present("genbank") {
        nwrdir.join("ar_genbank.sqlite")
    } else {
        nwrdir.join("ar_refseq.sqlite")
    };
    if file.exists() {
        std::fs::remove_file(&file).unwrap();
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
    let tx_conn = nwr::connect_txdb(&nwrdir).unwrap();

    info!("==> Create tables");
    conn.execute_batch(DDL_AR)?;

    info!("==> Loading...");
    let file = if args.is_present("genbank") {
        File::open(nwrdir.join("assembly_summary_genbank.txt"))?
    } else {
        File::open(nwrdir.join("assembly_summary_refseq.txt"))?
    };
    let rdr = BufReader::new(file);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, line) in rdr.lines().filter_map(|r| r.ok()).enumerate() {
        if line.starts_with("#") {
            continue;
        }

        if i > 1 && i % 1000 == 0 {
            stmts.push(String::from("COMMIT;"));
            let stmt = &stmts.join("\n");
            conn.execute_batch(stmt)?;
            stmts.clear();
            stmts.push(String::from("BEGIN;"));
        }
        if i > 1 && i % 100000 == 0 {
            debug!("Read {} records", i);
        }

        let fields: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();

        // fields
        let tax_id = fields.get(5).unwrap().parse::<i64>().unwrap();
        let organism_name = fields.get(7).unwrap();
        let bioproject = fields.get(1).unwrap();
        let assembly_accession = fields.get(0).unwrap();
        let refseq_category = fields.get(4).unwrap();
        let assembly_level = fields.get(11).unwrap();
        let genome_rep = fields.get(13).unwrap();
        let seq_rel_date = fields.get(14).unwrap();
        let asm_name = fields.get(15).unwrap();
        let ftp_path = fields.get(19).unwrap();

        // Skip incompetent strains
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?xi)\b(Candidatus|candidate|uncultured|unidentified|bacterium|archaeon|metagenome|virus|phage)\b").unwrap();
        }
        if RE.is_match(organism_name) {
            // debug!("Skip: {}", organism_name);
            continue;
        }

        // lineage
        let lineage = match nwr::get_lineage(&tx_conn, tax_id) {
            Err(err) => {
                warn!("Errors on get_lineage({}): {}", tax_id, err);
                let mut node: Node = Default::default();
                node.tax_id = 0;
                node.rank = "no rank".to_string();
                node.names = HashMap::from([("".to_string(), vec!["NA".to_string()])]);
                vec![node]
            }
            Ok(x) => x,
        };
        let (family_id, family) = nwr::find_rank(&lineage, "family".to_string());
        let (genus_id, genus) = nwr::find_rank(&lineage, "genus".to_string());
        let (species_id, species) = nwr::find_rank(&lineage, "species".to_string());

        // create stmt
        let stmt = format!(
            "INSERT INTO ar(
                tax_id, organism_name, bioproject, assembly_accession, refseq_category,
                assembly_level, genome_rep, seq_rel_date, asm_name, ftp_path,
                family, family_id, genus, genus_id, species, species_id
            )
            VALUES (
                    {},  '{}', '{}', '{}', '{}',
                    '{}', '{}', '{}', '{}', '{}',
                    '{}', {}, '{}', {}, '{}', {}
            );",
            tax_id.to_string(),
            organism_name.replace("'", "''"),
            bioproject,
            assembly_accession,
            refseq_category, // 5
            assembly_level,
            genome_rep,
            seq_rel_date.replace("/", "-"), // Transform seq_rel_date to SQLite Date format
            asm_name,
            ftp_path, // 10
            family.replace("'", "''"),
            family_id.to_string(),
            genus.replace("'", "''"),
            genus_id.to_string(),
            species.replace("'", "''"),
            species_id.to_string(),
        );
        stmts.push(stmt);
    }

    // There could left records in stmts
    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;

    debug!("Creating indexes for ar");
    conn.execute("CREATE INDEX idx_ar_tax_id ON ar(tax_id);", [])?;
    conn.execute("CREATE INDEX idx_ar_family ON ar(family);", [])?;
    conn.execute("CREATE INDEX idx_ar_family_id ON ar(family_id);", [])?;
    conn.execute("CREATE INDEX idx_ar_genus ON ar(genus);", [])?;
    conn.execute("CREATE INDEX idx_ar_genus_id ON ar(genus_id);", [])?;
    conn.execute("CREATE INDEX idx_ar_species ON ar(species);", [])?;
    conn.execute("CREATE INDEX idx_ar_species_id ON ar(species_id);", [])?;

    Ok(())
}

use clap::*;
use lazy_static::lazy_static;
use log::{debug, info};
use nwr::Taxon;
use regex::Regex;
use simplelog::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("ardb")
        .about("Init the assembly database")
        .after_help(
            r#"
This command init the assembly database, which includes metadata for assemblies on the NCBI genomes FTP site.

~/.nwr/ar_refseq.sqlite
~/.nwr/ar_genbank.sqlite

* `assembly_summary_*.txt` have 23 tab-delimited columns.
* Fields with numbers are used in the database.

    0   assembly_accession  6
    1   bioproject          4
    2   biosample           5
    3   wgs_master
    4   refseq_category     7
    5   taxid AS tax_id     1
    6   species_taxid
    7   organism_name       2
    8   infraspecific_name  3
    9   isolate
    10  version_status
    11  assembly_level      8
    12  release_type
    13  genome_rep          9
    14  seq_rel_date        10
    15  asm_name            11
    16  submitter
    17  gbrs_paired_asm     12
    18  paired_asm_comp
    19  ftp_path            13
    20  excluded_from_refseq
    21  relation_to_type_material
    22  asm_not_live_date

* 6 columns appended

    14  species
    15  species_id
    16  genus
    17  genus_id
    18  family
    19  family_id

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

* The DDL

    echo "
        SELECT sql
        FROM sqlite_master
        WHERE type='table'
        ORDER BY name;
        " |
        sqlite3 -tabs ~/.nwr/ar_refseq.sqlite

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
    )

* Requires SQLite version 3.34 or above.

"#,
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .value_name("DIR")
                .help("Change working directory"),
        )
        .arg(
            Arg::new("genbank")
                .long("genbank")
                .action(ArgAction::SetTrue)
                .help("Create the genbank ardb"),
        )
}

static DDL_AR: &str = r###"
DROP TABLE IF EXISTS ar;

CREATE TABLE IF NOT EXISTS ar (
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

"###;

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

    let nwrdir = if args.contains_id("dir") {
        std::path::Path::new(args.get_one::<String>("dir").unwrap()).to_path_buf()
    } else {
        nwr::nwr_path()
    };
    let file = if args.get_flag("genbank") {
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
    let file = if args.get_flag("genbank") {
        File::open(nwrdir.join("assembly_summary_genbank.txt"))?
    } else {
        File::open(nwrdir.join("assembly_summary_refseq.txt"))?
    };
    let rdr = BufReader::new(file);

    let mut stmts: Vec<String> = vec![String::from("BEGIN;")];
    for (i, line) in rdr.lines().map_while(Result::ok).enumerate() {
        if line.starts_with('#') {
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

        if fields.len() < 19 {
            continue;
        }

        // fields
        let tax_id = fields.get(5).unwrap().parse::<i64>().unwrap();
        let organism_name = fields.get(7).unwrap();
        let infraspecific_name = fields.get(8).unwrap();
        let bioproject = fields.get(1).unwrap();
        let biosample = fields.get(2).unwrap();
        let assembly_accession = fields.get(0).unwrap();
        let refseq_category = fields.get(4).unwrap();
        let assembly_level = fields.get(11).unwrap();
        let genome_rep = fields.get(13).unwrap();
        let seq_rel_date = fields.get(14).unwrap();
        let asm_name = fields.get(15).unwrap();
        let gbrs_paired_asm = fields.get(17).unwrap();
        let ftp_path = fields.get(19).unwrap();

        // Skip incompetent strains
        lazy_static! {
            static ref RE1: Regex = Regex::new(r"(?xi)\b(uncultured|unidentified|bacterium|archaeon|metagenome)\b").unwrap();
            static ref RE2: Regex = Regex::new(r"(?xi)(virus|phage)\b").unwrap();
        }
        if RE1.is_match(organism_name) || RE2.is_match(organism_name) {
            // debug!("Skip: {}", organism_name);
            continue;
        }

        // lineage
        let lineage = match nwr::get_lineage(&tx_conn, tax_id) {
            Err(err) => {
                debug!("Errors on get_lineage({}): {}", tax_id, err);
                let mut taxon: Taxon = Default::default();
                taxon.tax_id = 0;
                taxon.rank = "no rank".to_string();
                taxon.names = HashMap::from([("".to_string(), vec!["NA".to_string()])]);
                vec![taxon]
            }
            Ok(x) => x,
        };
        let (species_id, species) = nwr::find_rank(&lineage, "species".to_string());
        let (genus_id, genus) = nwr::find_rank(&lineage, "genus".to_string());
        let (family_id, family) = nwr::find_rank(&lineage, "family".to_string());

        // create stmt
        let stmt = format!(
            "INSERT INTO ar(
                tax_id, organism_name, infraspecific_name, bioproject, biosample, assembly_accession, refseq_category,
                assembly_level, genome_rep, seq_rel_date, asm_name, gbrs_paired_asm, ftp_path,
                species, species_id, genus, genus_id, family, family_id
            )
            VALUES (
                    {},  '{}', '{}', '{}', '{}', '{}', '{}',
                    '{}', '{}', '{}', '{}', '{}', '{}',
                    '{}', {}, '{}', {}, '{}', {}
            );",
            // 1-7
            tax_id,
            organism_name.replace('\'', "''"),
            infraspecific_name.replace('\'', "''"),
            bioproject,
            biosample,
            assembly_accession,
            refseq_category,
            // 8-13
            assembly_level,
            genome_rep,
            seq_rel_date.replace('/', "-"), // Transform seq_rel_date to SQLite Date format
            asm_name,
            gbrs_paired_asm,
            ftp_path,
            // 13-18
            species.replace('\'', "''"),
            species_id,
            genus.replace('\'', "''"),
            genus_id,
            family.replace('\'', "''"),
            family_id,
        );
        stmts.push(stmt);
    }

    // There could left records in stmts
    stmts.push(String::from("COMMIT;"));
    let stmt = &stmts.join("\n");
    conn.execute_batch(stmt)?;

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

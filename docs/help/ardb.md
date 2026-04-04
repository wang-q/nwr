# ardb

Behavior:

* Initializes the assembly database from assembly summary files.
* Creates SQLite databases at `~/.nwr/ar_refseq.sqlite` and `~/.nwr/ar_genbank.sqlite`.
* Loads data from `assembly_summary_refseq.txt` or `assembly_summary_genbank.txt`.
* Appends taxonomic lineage information (species, genus, family).
* Filters out incompetent strains (uncultured, unidentified, etc.).

Database Location:

    ~/.nwr/ar_refseq.sqlite
    ~/.nwr/ar_genbank.sqlite

Input Columns:

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

Appended Columns:

    14  species
    15  species_id
    16  genus
    17  genus_id
    18  family
    19  family_id

Filtered Strains:

Incompetent strains matching the following regexes in their `organism_name` are removed:

    \bCandidatus\b
    \bcandidate\b
    \buncultured\b
    \bunidentified\b
    \bbacterium\b
    \barchaeon\b
    \bmetagenome\b
    virus\b
    phage\b

Requirements:

* Strains with `assembly_level` of Scaffold or Contig should have a `genome_rep` of `full`.
* Requires SQLite version 3.34 or above.

Query the database:

    echo "
        SELECT
            COUNT(*)
        FROM ar
        WHERE 1=1
            AND genus IN ('Pseudomonas')
            AND assembly_level IN ('Complete Genome', 'Chromosome')
        " |
        sqlite3 -tabs ~/.nwr/ar_refseq.sqlite

The DDL:

```sql
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
```

Examples:

1. Initialize the RefSeq assembly database
   `nwr ardb`

2. Initialize the GenBank assembly database
   `nwr ardb --genbank`

3. Use a custom directory
   `nwr ardb --dir /path/to/nwr`

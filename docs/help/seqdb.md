# seqdb

Behavior:

* Initializes the sequence database for protein sequence information.
* Creates a SQLite database at `./seq.sqlite`.
* Loads data from various TSV files into appropriate tables.
* Supports loading strains, sizes, clusters, annotations, and assembly sequences.

Database Location:

    ./seq.sqlite

The DDL:

```sql
CREATE TABLE rank (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE
);
-- assembly
CREATE TABLE asm (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    rank_id INTEGER NOT NULL,
    FOREIGN KEY (rank_id) REFERENCES rank(id)
);
-- sequence
CREATE TABLE seq (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    size INTEGER,
    anno TEXT
);
-- representative
CREATE TABLE rep (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    f1 TEXT,
    f2 TEXT,
    f3 TEXT,
    f4 TEXT,
    f5 TEXT,
    f6 TEXT,
    f7 TEXT,
    f8 TEXT
);
-- Junction table to associate rep with seq
CREATE TABLE rep_seq (
    rep_id INTEGER NOT NULL,
    seq_id INTEGER NOT NULL,
    PRIMARY KEY (rep_id, seq_id),
    FOREIGN KEY (rep_id) REFERENCES rep(id),
    FOREIGN KEY (seq_id) REFERENCES seq(id)
);
-- Junction table to associate asm with seq
CREATE TABLE asm_seq (
    asm_id INTEGER NOT NULL,
    seq_id INTEGER NOT NULL,
    PRIMARY KEY (asm_id, seq_id),
    FOREIGN KEY (asm_id) REFERENCES asm(id),
    FOREIGN KEY (seq_id) REFERENCES seq(id)
);
-- Regular indices
CREATE INDEX rep_idx_f1 ON rep(f1);
CREATE INDEX rep_idx_f2 ON rep(f2);
CREATE INDEX rep_idx_f3 ON rep(f3);
CREATE INDEX rep_idx_f4 ON rep(f4);
CREATE INDEX rep_idx_f5 ON rep(f5);
CREATE INDEX rep_idx_f6 ON rep(f6);
CREATE INDEX rep_idx_f7 ON rep(f7);
CREATE INDEX rep_idx_f8 ON rep(f8);
-- Case-insensitive indices for `like`
CREATE INDEX seq_idx_anno ON seq(anno COLLATE NOCASE);
```

Notes:

* If `--strain` is called without specifying a path, it will load the default file under `--dir`.
* `--rep` requires a key-value pair in the format `--rep f1=file`.
* Valid fields for `--rep` are: f1, f2, f3, f4, f5, f6, f7, f8.

Examples:

1. Initialize the database
   `nwr seqdb --init`

2. Load strain information
   `nwr seqdb --strain strains.tsv`

3. Load multiple data types
   `nwr seqdb --strain --size --clust`

4. Load features into rep table
   `nwr seqdb --rep f1=features.tsv`

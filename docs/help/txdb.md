# txdb

Behavior:

* Initializes the taxonomy database from `taxdump.tar.gz`.
* Creates a SQLite database at `~/.nwr/taxonomy.sqlite`.
* Loads data from `division.dmp`, `names.dmp`, and `nodes.dmp`.
* Creates indexes for efficient querying.

Database Location:

    ~/.nwr/taxonomy.sqlite

The DDL:

```sql
DROP TABLE IF EXISTS division;
DROP TABLE IF EXISTS node;
DROP TABLE IF EXISTS name;

CREATE TABLE IF NOT EXISTS division (
    id       INTEGER      NOT NULL
                          PRIMARY KEY,
    division VARCHAR (50) NOT NULL
);

CREATE TABLE IF NOT EXISTS node (
    tax_id        INTEGER      NOT NULL
                               PRIMARY KEY,
    parent_tax_id INTEGER,
    rank          VARCHAR (25) NOT NULL,
    division_id   INTEGER      NOT NULL,
    comment       TEXT,
    FOREIGN KEY (
        division_id
    )
    REFERENCES division (id)
);

CREATE TABLE IF NOT EXISTS name (
    id         INTEGER      NOT NULL
                            PRIMARY KEY,
    tax_id     INTEGER      NOT NULL,
    name       VARCHAR (50) NOT NULL,
    name_class VARCHAR (50) NOT NULL
);
```

Query the database:

    echo "
        SELECT sql
        FROM sqlite_master
        WHERE type='table'
        ORDER BY name;
        " |
        sqlite3 -tabs ~/.nwr/taxonomy.sqlite

Examples:

1. Initialize the taxonomy database
   `nwr txdb`

2. Use a custom directory
   `nwr txdb --dir /path/to/nwr`

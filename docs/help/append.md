# append

Behavior:

* Retrieves taxonomic information from the local taxonomy database.
* Appends scientific names and/or taxon IDs of specified ranks to each row.
* If `--rank` is not specified, appends the scientific name of the input taxon.
* Header lines (starting with "#") are processed to append appropriate column names.

Valid ranks:

* species, genus, family, order, class, phylum, kingdom
* Other ranks (e.g., clade, no rank) may work but are not officially supported.

Input:

* Accepts one or more TSV files as input.
* Reads from standard input if "stdin" is specified.
* The input file should contain taxon IDs or scientific names in a specific column.

Output:

* Tab-separated values with appended rank columns.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Append scientific names for specified ranks
   `nwr append input.tsv --rank genus --rank family`

2. Append both names and IDs
   `nwr append input.tsv --rank species --id`

3. Read from stdin, append genus information
   `cat input.tsv | nwr append stdin --rank genus`

4. Specify column and output file
   `nwr append input.tsv -c 2 --rank kingdom -o output.tsv`

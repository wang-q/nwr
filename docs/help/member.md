# member

Behavior:

* Lists members (of certain ranks) under ancestral term(s).
* Retrieves taxonomic information from the local taxonomy database.
* Ancestral terms can be Taxonomy IDs or scientific names.
* By default, excludes "Environmental samples" division.
* The output file is in the same TSV format as `nwr info --tsv`.

Valid ranks:

* species, genus, family, order, class, phylum, kingdom
* Other ranks (e.g., clade, no rank) may work but are not officially supported.

Input:

* Accepts one or more ancestral Taxonomy IDs or scientific names.
* Optionally filter results by rank using `--rank`.

Output:

* TSV output includes: tax_id, sci_name, rank, division.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. List all members under a genus
   `nwr member 9605`

2. List only species under a genus
   `nwr member Homo --rank species`

3. Include environmental samples
   `nwr member 4751 --env`

4. Multiple ancestors with rank filter
   `nwr member Homo Pan --rank genus`

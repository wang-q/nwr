# info

Behavior:

* Retrieves taxonomic information from the local taxonomy database.
* Accepts Taxonomy IDs or scientific names as input.
* By default, outputs detailed information in a custom format.
* Use `--tsv` to output results as tab-separated values.

Input:

* Accepts one or more Taxonomy IDs or scientific names.
* Terms can be provided as positional arguments.

Output:

* Default format shows detailed taxonomic information.
* TSV output includes: tax_id, sci_name, rank, division.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Get information for a single taxon
   `nwr info 9606`

2. Get information for multiple taxa
   `nwr info 9606 10090 10116`

3. Output as TSV
   `nwr info Homo_sapiens --tsv`

4. Use scientific names
   `nwr info "Homo sapiens" "Mus musculus"`

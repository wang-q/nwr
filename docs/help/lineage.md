# lineage

Behavior:

* Retrieves the lineage of a taxon from root to the specified term.
* Returns the full taxonomic hierarchy including all ranks.
* By default, outputs rank, scientific name, and taxonomy ID for each level.

Input:

* Accepts a single Taxonomy ID or scientific name.
* Use `--tsv` for tab-separated output format.

Output:

* Default output: rank, scientific_name, tax_id (tab-separated)
* TSV output: rank, scientific_name, tax_id (tab-separated)
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Get lineage for a species
   `nwr lineage 9606`

2. Get lineage using scientific name
   `nwr lineage "Homo sapiens"`

3. Output as TSV
   `nwr lineage 9606 --tsv`

4. Write to file
   `nwr lineage 9606 -o lineage.txt`

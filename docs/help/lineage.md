# lineage

Behavior:

* Retrieves the lineage of a taxon from root to the specified term.
* Returns the full taxonomic hierarchy including all ranks.
* Outputs rank, scientific name, and taxonomy ID for each level.

Input:

* Accepts a single Taxonomy ID or scientific name.

Output:

* Output is tab-separated: rank, scientific_name, tax_id.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Get lineage for a species
   `nwr lineage 9606`

2. Get lineage using scientific name
   `nwr lineage "Homo sapiens"`

3. Write to file
   `nwr lineage 9606 -o lineage.txt`

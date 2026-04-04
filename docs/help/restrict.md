# restrict

Behavior:

* Restricts taxonomy terms to descendants of specified ancestor(s).
* Terms can be Taxonomy IDs or scientific names.
* Use `--exclude` to invert the filter (exclude matching lines).
* Header lines (starting with "#") are always outputted.

Input:

* Accepts one or more TSV files via `--file` option.
* Reads from standard input by default.
* The input file should contain taxon IDs or scientific names in a specific column.

Output:

* Filtered tab-separated values.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Restrict to descendants of a specific genus
   `nwr restrict "Homo" --file input.tsv`

2. Restrict using taxonomy ID
   `nwr restrict 9605 --file input.tsv`

3. Exclude descendants (inverse filter)
   `nwr restrict "Bacteria" --file input.tsv --exclude`

4. Specify column and output file
   `nwr restrict "Mammalia" --file input.tsv -c 2 -o output.tsv`

5. Multiple ancestors
   `nwr restrict "Homo" "Pan" --file input.tsv`

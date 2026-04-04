# common

Behavior:

* Outputs the common tree of terms as Newick format.
* Finds the most recent common ancestor of all input terms.
* Constructs a phylogenetic tree showing the relationship.
* Ancestral terms can be Taxonomy IDs or scientific names.

Input:

* Accepts two or more Taxonomy IDs or scientific names.
* Terms are provided as positional arguments.

Output:

* Newick format tree string.
* Tree includes scientific names as node labels.
* By default, output is written to standard output.
* Use `--outfile` to write to a file instead.

Examples:

1. Find common ancestor of two species
   `nwr common 9606 10090`

2. Find common ancestor of multiple taxa
   `nwr common "Homo sapiens" "Mus musculus" "Danio rerio"`

3. Write to file
   `nwr common 9606 10090 -o tree.nwk`

4. Use taxonomy IDs
   `nwr common 9605 10090 10116`

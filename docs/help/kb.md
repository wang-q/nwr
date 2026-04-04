# kb

Behavior:

* Prints embedded documentation and knowledge bases.
* Extracts built-in files to stdout or a specified output directory.

Available Documents:

* `bac120` - 120 bacterial marker genes (tar.gz archive)
* `ar53` - 53 archaeal marker genes (tar.gz archive)

Output:

* Archive files (bac120, ar53) are extracted to the specified directory.
* By default, output is written to standard output.
* Use `--outfile` to specify an output file or directory.

Examples:

1. Extract bacterial marker genes
   `nwr kb bac120 -o marker_genes/`

2. Extract archaeal marker genes
   `nwr kb ar53 -o marker_genes/`

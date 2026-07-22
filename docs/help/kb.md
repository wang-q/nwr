# kb

Behavior:

* Prints embedded documentation and knowledge bases.
* Extracts built-in files to stdout or a specified output directory.

Available Documents:

* `bac120` - 120 bacterial marker genes (tar.gz archive)
* `ar53` - 53 archaeal marker genes (tar.gz archive)

Output:

* Archive files (bac120, ar53) are extracted to the specified directory.
* By default, files are extracted to the current directory.
* Use `--outdir` to specify an output directory.

Examples:

1. Extract bacterial marker genes
   `nwr kb bac120 --outdir marker_genes/`

2. Extract archaeal marker genes
   `nwr kb ar53 --outdir marker_genes/`

# template

Behavior:

* Creates directories, data files, and scripts for phylogenomic research.
* Generates materials for ASSEMBLY, BioSample, MinHash, Count, and Protein steps.
* Uses Tera templates to generate Bash scripts.

Input File Format:

`.assembly.tsv` is a TAB-delimited file to guide downloading and processing:

| Col |  Type  | Description                                              |
|----:|:------:|:---------------------------------------------------------|
|   1 | string | #name: species + infraspecific_name + assembly_accession |
|   2 | string | ftp_path                                                 |
|   3 | string | biosample                                                |
|   4 | string | species                                                  |
|   5 | string | assembly_level                                           |

Generated Materials:

* `--ass`: ASSEMBLY/
    * One TSV file: url.tsv
    * Five Bash scripts: rsync.sh, check.sh, n50.sh, collect.sh, finish.sh

* `--bs`: BioSample/
    * One TSV file: sample.tsv
    * Two Bash scripts: download.sh, collect.sh

* `--mh`: MinHash/
    * One TSV file: species.tsv
    * Five Bash scripts: compute.sh, species.sh, abnormal.sh, nr.sh, dist.sh

* `--count`: Count/
    * One TSV file: species.tsv
    * Three Bash scripts: strains.sh, rank.sh, lineage.sh

* `--pro`: Protein/
    * One TSV file: species.tsv
    * Bash scripts: collect.sh, info.sh, count.sh

Examples:

1. Generate ASSEMBLY materials
   `nwr template input.assembly.tsv --ass`

2. Generate all materials
   `nwr template input.assembly.tsv --ass --bs --mh --count --pro`

3. Specify output directory
   `nwr template input.assembly.tsv --ass -o output_dir`

4. Use parallel processing
   `nwr template input.assembly.tsv --mh --parallel 16`

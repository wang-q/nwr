# nwr

[![Publish](https://github.com/wang-q/nwr/actions/workflows/publish.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Build](https://github.com/wang-q/nwr/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Codecov](https://img.shields.io/codecov/c/github/wang-q/nwr/master.svg)](https://codecov.io/github/wang-q/nwr?branch=master)
[![Crates.io](https://img.shields.io/crates/v/nwr.svg)](https://crates.io/crates/nwr)
[![Lines of code](https://tokei.rs/b1/github/wang-q/nwr?category=code)](https://github.com//wang-q/nwr)

`nwr` is a command line tool for working with **N**CBI taxonomy, Ne**w**ick files and assembly
**r**eports, written in Rust.

## Install

Current release: 0.6.3

```shell
cargo install nwr

# or
brew install wang-q/tap/nwr

# local repo
cargo install --path . --force --offline

# build under WSL 2
export CARGO_TARGET_DIR=/tmp
cargo build

```

## `nwr help`

```text
`nwr` is a command line tool for working with NCBI taxonomy, Newick files and assembly reports

Usage: nwr [COMMAND]

Commands:
  append    Append fields of higher ranks to a TSV file
  ardb      Init the assembly database
  comment   Add comments to node(s) in a Newick file
  download  Download the latest releases of `taxdump` and assembly reports
  indent    Indent the Newick file
  info      Information of Taxonomy ID(s) or scientific name(s)
  kb        Prints docs (knowledge bases)
  label     Labels in the Newick file
  lineage   Output the lineage of the term
  member    List members (of certain ranks) under ancestral term(s)
  order     Order nodes in a Newick file
  rename    Rename node(s) in a Newick file
  restrict  Restrict taxonomy terms to ancestral descendants
  stat      Statistics about the Newick file
  template  Create dirs, data and scripts for a phylogenomic research
  tex       Visualize the Newick tree via LaTeX
  txdb      Init the taxonomy database
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


Subcommand groups:

* Database
    * download
    * txdb
    * ardb

* Taxonomy
    * info
    * lineage
    * member
    * append
    * restrict

* Newick
    * indent
    * order
    * label
    * rename
    * comment
    * stat
    * tex

* Assembly
    * template
    * kb

```

## Examples

### Usage of each command

For practical uses of `nwr` and other awesome companions, follow this [page](doc/ncbi_ar.md).

```shell
nwr download

nwr txdb

nwr info "Homo sapiens" 4932

nwr lineage "Homo sapiens"
nwr lineage 4932

nwr restrict "Vertebrata" -c 2 -f tests/nwr/taxon.tsv
##sci_name       tax_id
#Human   9606

nwr member "Homo"

nwr append tests/nwr/taxon.tsv -c 2 -r species -r family --id

nwr ardb
nwr ardb --genbank

```

### Development

```shell
# Concurrent tests may trigger sqlite locking
cargo test -- --test-threads=1

cargo test --color=always --package nwr --test cli_nwr command_template -- --show-output

# debug mode has a slow connection
cargo run --release --bin nwr download

# tests/nwr/
cargo run --bin nwr txdb -d tests/nwr/

cargo run --bin nwr info -d tests/nwr/ --tsv Viruses "Actinophage JHJ-1" "Bacillus phage bg1"

cargo run --bin nwr lineage -d tests/nwr/ --tsv "Actinophage JHJ-1"

echo -e '#ID\n9606\n12347' |
    cargo run --bin nwr restrict -d tests/nwr/ "Viruses"
cargo run --bin nwr restrict -d tests/nwr/ "Viruses" -c 2 -f tests/nwr/taxon.tsv -f tests/nwr/taxon.tsv
cargo run --bin nwr restrict -d tests/nwr/ "Viruses" -c 2 -f tests/nwr/taxon.tsv -e

cargo run --bin nwr member -d tests/nwr/ "Synechococcus phage S" -r "no rank" -r species
cargo run --bin nwr member -d tests/nwr/ "Synechococcus phage S"

echo -e '#tax_id\n12347' |
    cargo run --bin nwr append -d tests/nwr/ stdin -r species -r family --id
cargo run --bin nwr append -d tests/nwr/ tests/nwr/taxon-valid.tsv -c 2 -r species -r family --id

cargo run --bin nwr ardb -d tests/nwr/

cargo run --bin nwr assembly tests/assembly/Trichoderma.assembly.tsv

```

### Newick files and LaTeX

For more detailed usages, check [this](tree/README.md).

```shell
nwr indent tests/newick/hg38.7way.nwk --text ".   "

echo "((A,B),C);" | nwr order --ndr stdin
nwr order --nd tests/newick/hg38.7way.nwk

nwr label tests/newick/hg38.7way.nwk

nwr rename tests/newick/abc.nwk -n C -r F -l A,B -r D

nwr stat tests/newick/hg38.7way.nwk
nw_stats tests/newick/hg38.7way.nwk

cargo run --bin nwr subtree tests/newick/hg38.7way.nwk -n Human -n Rhesus -r "^ch" -m

cargo run --bin nwr subtree tests/newick/hg38.7way.nwk -n Human -n Rhesus
nw_clade tests/newick/hg38.7way.nwk Human Rhesus

nw_topology tests/newick/hg38.7way.nwk

nw_reroot tests/newick/catarrhini_wrong.nwk Cebus |
    nw_order -c n -
perl doc/reroot.pl tests/newick/catarrhini_wrong.nwk Cebus |
    nw_order -c n -

echo "((A,B),C);" |
    nwr comment stdin -n A -n C --color green -s '&&nwr' |
    nwr comment stdin -l A,B --dot

latexmk -xelatex doc/template.tex
latexmk -c doc/template.tex

cargo run --bin nwr tex --bare tests/newick/hg38.7way.nwk

cargo run --bin nwr tex --bl tests/newick/hg38.7way.nwk -o output.tex
latexmk -xelatex output.tex
latexmk -c output.tex

cargo run --bin nwr tex --forest --bare tests/newick/test.forest

```

## Database schema

```shell
brew install k1LoW/tap/tbls

tbls doc sqlite://./tests/nwr/taxonomy.sqlite doc/txdb

tbls doc sqlite://./tests/nwr/ar_refseq.sqlite doc/ardb

```

[txdb](./doc/txdb/README.md)

[ardb](./doc/ardb/README.md)

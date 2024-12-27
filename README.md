# nwr

[![Publish](https://github.com/wang-q/nwr/actions/workflows/publish.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Build](https://github.com/wang-q/nwr/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Codecov](https://img.shields.io/codecov/c/github/wang-q/nwr/master.svg)](https://codecov.io/github/wang-q/nwr?branch=master)
[![Crates.io](https://img.shields.io/crates/v/nwr.svg)](https://crates.io/crates/nwr)
![](https://img.shields.io/crates/d/nwr?label=downloads%20%28crates.io%29)
[![Lines of code](https://www.aschey.tech/tokei/github/wang-q/nwr)](https://github.com//wang-q/nwr)

`nwr` is a command line tool for working with **N**CBI taxonomy, Ne**W**ick files and assembly
**R**eports, written in Rust.

## Install

Current release: 0.7.7

```shell
cargo install nwr

# or
brew install wang-q/tap/nwr

cargo install --path . --force # --offline

# Concurrent tests may trigger sqlite locking
cargo test -- --test-threads=1

# build under WSL 2
mkdir -p /tmp/cargo
export CARGO_TARGET_DIR=/tmp/cargo
cargo build

# build for CentOS 7
# rustup target add x86_64-unknown-linux-gnu
# pip3 install cargo-zigbuild
cargo zigbuild --target x86_64-unknown-linux-gnu.2.17 --release
ll $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/

```

## `nwr help`

```text
`nwr` is a command line tool for working with NCBI taxonomy, Newick files and assembly reports

Usage: nwr [COMMAND]

Commands:
  append       Append fields of higher ranks to a TSV file
  ardb         Init the assembly database
  comment      Add comments to node(s) in a Newick file
  common       Output the common tree of terms
  distance     Output a TSV/phylip file with distances between all named nodes
  download     Download the latest releases of `taxdump` and assembly reports
  indent       Indent the Newick file
  info         Information of Taxonomy ID(s) or scientific name(s)
  kb           Prints docs (knowledge bases)
  label        Labels in the Newick file
  lineage      Output the lineage of the term
  member       List members (of certain ranks) under ancestral term(s)
  order        Order nodes in a Newick file
  pl-condense  Pipeline - condense subtrees based on taxonomy
  prune        Remove nodes from the Newick file
  rename       Rename named/unnamed nodes in a Newick file
  replace      Replace node names/comments in a Newick file
  reroot       Place the root in the middle of the desired node and its parent
  restrict     Restrict taxonomy terms to ancestral descendants
  seqdb        Init the seq database
  subtree      Extract a subtree
  stat         Statistics about the Newick file
  template     Create dirs, data and scripts for a phylogenomic research
  tex          Visualize the Newick tree via LaTeX
  topo         Topological information of the Newick file
  txdb         Init the taxonomy database
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


Subcommand groups:

* Database
    * download / txdb / ardb

* Taxonomy
    * info / lineage / member / append / restrict / common

* Newick
    * Information
        * label / stat / distance
    * Manipulation
        * order / rename / replace / topo / subtree / prune / reroot
        * pl-condense
    * Visualization
        * indent / comment / tex

* Assembly
    * template
    * kb
    * seqdb

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

nwr common "Escherichia coli" 4932 Drosophila_melanogaster 9606 "Mus musculus"

# rm ~/.nwr/*.dmp

```

### Development

```shell
cargo test --color=always --package nwr --test cli_nwr command_template -- --show-output

# debug mode has a slow connection
cargo run --release --bin nwr download

# tests/nwr/
cargo run --bin nwr txdb -d tests/nwr/

cargo run --bin nwr info -d tests/nwr/ --tsv Viruses "Actinophage JHJ-1" "Bacillus phage bg1"

cargo run --bin nwr common -d tests/nwr/ "Actinophage JHJ-1" "Bacillus phage bg1"

cargo run --bin nwr template tests/assembly/Trichoderma.assembly.tsv --ass -o stdout

```

### seqdb

```shell
export SPECIES="$HOME/data/Archaea/Protein/Sulfolobus_acidocaldarius"

cargo run --bin nwr seqdb -d ${SPECIES} --init --strain

cargo run --bin nwr seqdb -d ${SPECIES} \
    --size <(
        hnsm size ${SPECIES}/pro.fa.gz
    ) \
    --clust

cargo run --bin nwr seqdb -d ${SPECIES} \
    --anno <(
        gzip -dcf "${SPECIES}"/anno.tsv.gz
    ) \
    --asmseq <(
        gzip -dcf "${SPECIES}"/asmseq.tsv.gz
    )

cargo run --bin nwr seqdb -d ${SPECIES} --rep f1="${SPECIES}"/fam88_cluster.tsv

echo "
    SELECT
        *
    FROM asm
    WHERE 1=1
    " |
    sqlite3 -tabs ${SEQ_DIR}/seq.sqlite

echo "
    SELECT
        COUNT(distinct asm_seq.asm_id)
    FROM asm_seq
    WHERE 1=1
    " |
    sqlite3 -tabs ${SEQ_DIR}/seq.sqlite

echo "
.header ON
    SELECT
        'species' AS species,
        COUNT(distinct asm_seq.asm_id) AS strain,
        COUNT(*) AS total,
        COUNT(distinct rep_seq.seq_id) AS dedup,
        COUNT(distinct rep_seq.rep_id) AS rep
    FROM asm_seq
    JOIN rep_seq ON asm_seq.seq_id = rep_seq.seq_id
    WHERE 1=1
    " |
    sqlite3 -tabs ${SEQ_DIR}/seq.sqlite


```

### Newick files and LaTeX

For more detailed usages, check [this file](tree/README.md).

#### Get information from the tree

```shell
# List all names
nwr label tests/newick/hg38.7way.nwk

# The intersection between the nodes in the tree and the provided
nwr label tests/newick/hg38.7way.nwk -r "^ch" -n Mouse -n foo
nwr label tests/newick/catarrhini.nwk -n Homo -n Pan -n Gorilla -M
# Is Pongo the sibling of Homininae?
nwr label tests/newick/catarrhini.nwk -n Homininae -n Pongo -DM
# All leaves belong to Hominidae
nwr label tests/newick/catarrhini.nwk -t Hominidae -I

nwr label tests/newick/catarrhini.nwk -c dup
nwr label tests/newick/catarrhini.comment.nwk -c full

nwr stat tests/newick/hg38.7way.nwk

# Various distances
nwr distance -m root -I tests/newick/catarrhini.nwk
nwr distance -m parent -I tests/newick/catarrhini.nwk
nwr distance -m pairwise -I tests/newick/catarrhini.nwk
nwr distance -m lca -I tests/newick/catarrhini.nwk

nwr distance -m root -L tests/newick/catarrhini_topo.nwk

# Phylip distance matrix
nwr distance -m phylip tests/newick/catarrhini.nwk

```

#### Manipulation of the tree

```shell
echo "((A,B),C);" | nwr order --ndr stdin
nwr order --nd tests/newick/hg38.7way.nwk

nwr rename tests/newick/abc.nwk -n C -r F -l A,B -r D

nwr replace tests/newick/abc.nwk tests/newick/abc.replace.tsv
nwr replace tests/newick/abc.nwk tests/newick/abc3.replace.tsv

nwr topo tests/newick/catarrhini.nwk

# The behavior is very similar to `nwr label`, but outputs a subtree instead of labels
nwr subtree tests/newick/hg38.7way.nwk -n Human -n Rhesus -r "^ch" -M

# Condense the subtree to a node
nwr subtree tests/newick/hg38.7way.nwk -n Human -n Rhesus -r "^ch" -M -c Primates

nwr subtree tests/newick/catarrhini.nwk -t Hominidae

nwr prune tests/newick/catarrhini.nwk -n Homo -n Pan

echo "((A:1,B:1)D:1,C:1)E;" |
    nwr reroot stdin -n B
nwr reroot tests/newick/catarrhini_wrong.nwk -n Cebus

nwr reroot tests/newick/bs.nw -n C

nwr tex tests/newick/bs.nw | tectonic -
mv texput.pdf bs.pdf
nwr reroot tests/newick/bs.nw -n C | nwr tex stdin | tectonic -
mv texput.pdf bs.reroot.pdf

cargo run --bin nwr pl-condense tests/newick/catarrhini.nwk -r family

```

#### Visualization of the tree

```shell
nwr indent tests/newick/hg38.7way.nwk --text ".   "

echo "((A,B),C);" |
    nwr comment stdin -n A -n C --color green |
    nwr comment stdin -l A,B --dot

tectonic doc/template.tex

echo "((A[color=green],B)[dot=black],C[color=green]);" |
    cargo run --bin nwr comment stdin -r "color="

nwr tex tests/newick/catarrhini.nwk -o output.tex
tectonic output.tex

nwr tex --bl tests/newick/hg38.7way.nwk

nwr tex --forest --bare tests/newick/test.forest

nwr common "Escherichia coli" 4932 Drosophila_melanogaster 9606 "Mus musculus" |
    nwr tex --bare stdin

```

## Database schema

```shell
brew install k1LoW/tap/tbls

tbls doc sqlite://./tests/nwr/taxonomy.sqlite doc/txdb

tbls doc sqlite://./tests/nwr/ar_refseq.sqlite doc/ardb

```

[txdb](./doc/txdb/README.md)

[ardb](./doc/ardb/README.md)

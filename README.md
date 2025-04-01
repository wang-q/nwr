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

Current release: 0.8.1

```shell
cargo install nwr

# or
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

```console
$ nwr help
`nwr` is a command line tool for working with NCBI taxonomy, Newick files and assembly reports

Usage: nwr [COMMAND]

Commands:
  download     Download the latest releases of `taxdump` and assembly reports
  txdb         Init the taxonomy database
  ardb         Init the assembly database
  append       Append fields of higher ranks to a TSV file
  common       Output the common tree of terms
  info         Information of Taxonomy ID(s) or scientific name(s)
  kb           Prints docs (knowledge bases)
  lineage      Output the lineage of the term
  member       List members (of certain ranks) under ancestral term(s)
  pl-condense  Pipeline - condense subtrees based on taxonomy
  restrict     Restrict taxonomy terms to ancestral descendants
  seqdb        Init the seq database
  template     Create dirs, data and scripts for a phylogenomic research
  data         Newick data commands
  ops          Newick operation commands
  viz          Newick visualization commands
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version


Subcommand groups:

* Database
    * download / txdb / ardb

* Taxonomy
    * info / lineage / member / append / restrict / common

* Assembly
    * template
    * kb
    * seqdb

* Newick
    * data
        * label / stat / distance
    * ops (operation)
        * order / rename / replace / topo / subtree / prune / reroot
        * pl-condense
    * viz (visualization)
        * indent / comment / tex

$ nwr data help
Newick data commands

Usage: nwr data <COMMAND>

Commands:
  label     Labels in the Newick file
  stat      Statistics about the Newick file
  distance  Output a TSV/phylip file with distances between all named nodes
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ nwr ops help
Newick operation commands

Usage: nwr ops <COMMAND>

Commands:
  order    Order nodes in a Newick file
  rename   Rename named/unnamed nodes in a Newick file
  replace  Replace node names/comments in a Newick file
  subtree  Extract a subtree
  topo     Topological information of the Newick file
  prune    Remove nodes from the Newick file
  reroot   Place the root in the middle of the desired node and its parent
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ nwr viz help
Newick visualization commands

Usage: nwr viz <COMMAND>

Commands:
  indent   Indent the Newick file
  comment  Add comments to node(s) in a Newick file
  tex      Visualize the Newick tree via LaTeX
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

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

nwr common "Escherichia coli" 4932 Drosophila_melanogaster 9606 Mus_musculus

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

#### Get data from the tree

```shell
# List all names
nwr data label tests/newick/hg38.7way.nwk

# The intersection between the nodes in the tree and the provided
nwr data label tests/newick/hg38.7way.nwk -r "^ch" -n Mouse -n foo
nwr data label tests/newick/catarrhini.nwk -n Homo -n Pan -n Gorilla -M
# Is Pongo the sibling of Homininae?
nwr data label tests/newick/catarrhini.nwk -n Homininae -n Pongo -DM
# All leaves belong to Hominidae
nwr data label tests/newick/catarrhini.nwk -t Hominidae -I

nwr data label tests/newick/catarrhini.nwk -c dup
nwr data label tests/newick/catarrhini.comment.nwk -c full

nwr data stat tests/newick/hg38.7way.nwk

# Various distances
nwr data distance -m root -I tests/newick/catarrhini.nwk
nwr data distance -m parent -I tests/newick/catarrhini.nwk
nwr data distance -m pairwise -I tests/newick/catarrhini.nwk
nwr data distance -m lca -I tests/newick/catarrhini.nwk

nwr data distance -m root -L tests/newick/catarrhini_topo.nwk

# Phylip distance matrix
nwr data distance -m phylip tests/newick/catarrhini.nwk

```

#### Operations of the tree

```shell
echo "((A,B),C);" | nwr ops order --ndr stdin
nwr ops order --nd tests/newick/hg38.7way.nwk

nwr ops order --list tests/newick/abcde.list tests/newick/abcde.nwk

nwr ops rename tests/newick/abc.nwk -n C -r F -l A,B -r D

nwr ops replace tests/newick/abc.nwk tests/newick/abc.replace.tsv
nwr ops replace tests/newick/abc.nwk tests/newick/abc3.replace.tsv

nwr ops topo tests/newick/catarrhini.nwk

# The behavior is very similar to `nwr label`, but outputs a subtree instead of labels
nwr ops subtree tests/newick/hg38.7way.nwk -n Human -n Rhesus -r "^ch" -M

# Condense the subtree to a node
nwr ops subtree tests/newick/hg38.7way.nwk -n Human -n Rhesus -r "^ch" -M -c Primates

nwr ops subtree tests/newick/catarrhini.nwk -t Hominidae

nwr ops prune tests/newick/catarrhini.nwk -n Homo -n Pan

echo "((A:1,B:1)D:1,C:1)E;" |
    nwr ops reroot stdin -n B
nwr ops reroot tests/newick/catarrhini_wrong.nwk -n Cebus

nwr ops reroot tests/newick/bs.nw -n C

nwr viz tex tests/newick/bs.nw | tectonic -
mv texput.pdf bs.pdf
nwr ops reroot tests/newick/bs.nw -n C | nwr viz tex stdin | tectonic -
mv texput.pdf bs.reroot.pdf

cargo run --bin nwr pl-condense tests/newick/catarrhini.nwk -r family

```

#### Visualization of the tree

```shell
nwr viz indent tests/newick/hg38.7way.nwk --text ".   "

echo "((A,B),C);" |
    nwr viz comment stdin -n A -n C --color green |
    nwr viz comment stdin -l A,B --dot

tectonic doc/template.tex

echo "((A[color=green],B)[dot=black],C[color=green]);" |
    cargo run --bin nwr viz comment stdin -r "color="

nwr viz tex tests/newick/catarrhini.nwk -o output.tex
tectonic output.tex

nwr viz tex --bl tests/newick/hg38.7way.nwk

nwr viz tex --forest --bare tests/newick/test.forest

nwr viz common "Escherichia coli" 4932 Drosophila_melanogaster 9606 "Mus musculus" |
    nwr viz tex --bare stdin

```

## Database schema

```shell
brew install k1LoW/tap/tbls

tbls doc sqlite://./tests/nwr/taxonomy.sqlite doc/txdb

tbls doc sqlite://./tests/nwr/ar_refseq.sqlite doc/ardb

```

[txdb](./doc/txdb/README.md)

[ardb](./doc/ardb/README.md)

# nwr

[![Publish](https://github.com/wang-q/nwr/actions/workflows/publish.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Build](https://github.com/wang-q/nwr/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Codecov](https://img.shields.io/codecov/c/github/wang-q/nwr/master.svg)](https://codecov.io/github/wang-q/nwr?branch=master)
[![Crates.io](https://img.shields.io/crates/v/nwr.svg)](https://crates.io/crates/nwr)
[![Lines of code](https://tokei.rs/b1/github/wang-q/nwr?category=code)](https://github.com//wang-q/nwr)

`nwr` is a command line tool for NCBI taxonomy and newick files.

## Install

Current release: 0.5.7

```shell
cargo install nwr

# or
brew install wang-q/tap/nwr

```

## SYNOPSIS

```text
$ nwr 0.5.7
wang-q <wang-q@outlook.com>
`nwr` is a command line tool for NCBI taxonomy and newick files

USAGE:
    nwr.exe [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    append      Append fields of higher ranks to a TSV file
    ardb        Init the assembly database
    download    Download the latest release of `taxdump`
    help        Print this message or the help of the given subcommand(s)
    info        Information of Taxonomy ID(s) or scientific name(s)
    lineage     Output the lineage of the term
    member      List members (of certain ranks) under ancestral term(s)
    restrict    Restrict taxonomy terms to ancestral descendants
    txdb        Init the taxonomy database

```

## EXAMPLES

### Usage of each command

For a realworld usage of `nwr` and other awesome campanions, follow this [page](doc/assembly.md).

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

# debug mode has a slow connection
cargo run --release --bin nwr download

# tests/nwr/
cargo run --bin nwr txdb -d tests/nwr/

cargo run --bin nwr info -d tests/nwr/ --tsv Viruses "Actinophage JHJ-1" "Bacillus phage bg1"

cargo run --bin nwr lineage -d tests/nwr/ --tsv "Actinophage JHJ-1"

echo -e '#ID\n9606\n12347' |
    cargo run --bin nwr restrict -d tests/nwr/ "Viruses"
cargo run --bin nwr restrict -d tests/nwr/ "Viruses" -c 2 -f tests/nwr/taxon.tsv -f tests/nwr/taxon.tsv

cargo run --bin nwr member -d tests/nwr/ "Synechococcus phage S" -r "no rank" -r species
cargo run --bin nwr member -d tests/nwr/ "Synechococcus phage S"

echo -e '#tax_id\n12347' |
    cargo run --bin nwr append -d tests/nwr/ stdin -r species -r family --id
cargo run --bin nwr append -d tests/nwr/ tests/nwr/taxon-valid.tsv -c 2 -r species -r family --id

cargo run --bin nwr ardb -d tests/nwr/

```

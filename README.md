# nwr - **N**CBI taxonomy/assembly **WR**angler

[![Publish](https://github.com/wang-q/nwr/actions/workflows/publish.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Build](https://github.com/wang-q/nwr/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/nwr/actions)
[![Codecov](https://img.shields.io/codecov/c/github/wang-q/nwr/master.svg)](https://codecov.io/github/wang-q/nwr?branch=master)
[![Crates.io](https://img.shields.io/crates/v/nwr.svg)](https://crates.io/crates/nwr)
![](https://img.shields.io/crates/d/nwr?label=downloads%20%28crates.io%29)
[![Lines of code](https://www.aschey.tech/tokei/github/wang-q/nwr)](https://github.com//wang-q/nwr)
[![Documentation](https://img.shields.io/badge/docs-online-blue)](https://wang-q.github.io/nwr/)

## Install

Current release: 0.9.0

```shell
cargo install nwr

# or
cargo install --path . --force # --offline
```

Or install the pre-compiled binary via the cross-platform package
manager [cbp](https://github.com/wang-q/cbp) (supports older Linux systems with glibc 2.17+):

```bash
cbp install nwr
```

You can also download the pre-compiled binaries from
the [Releases](https://github.com/wang-q/nwr/releases) page.

## `nwr help`

```console
$ nwr help
`nwr` is a command line **N**CBI taxonomy and assembly **WR**angler.

Usage: nwr [COMMAND]

Commands:
  download     Download the latest releases of `taxdump` and assembly reports
  txdb         Init the taxonomy database
  ardb         Init the assembly database
  info         Information of Taxonomy ID(s) or scientific name(s)
  lineage      Output the lineage of the term
  member       List members (of certain ranks) under ancestral term(s)
  append       Append fields of higher ranks to a TSV file
  restrict     Restrict taxonomy terms to ancestral descendants
  common       Output the common tree of terms
  template     Create dirs, data and scripts for a phylogenomic research
  kb           Prints docs (knowledge bases)
  seqdb        Init the seq database
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
    * template / kb / seqdb
```

## Examples

### Initiate local databases

The date `date --utc` of executing `nwr download` is `Sun Apr  5 15:59:45 UTC 2026`

The database doesn't need frequent updates. In our lab, we update it approximately once a year. For reproducibility, I provide database files for the above date in [the Releases](https://github.com/wang-q/nwr/releases/tag/v0.9.0) page.

```shell
cbp install nwr

nwr download
nwr txdb

nwr ardb
nwr ardb --genbank

cd $HOME/.nwr
tar cvfz ncbi.$(date +"%Y%m%d").tar.gz \
    taxdump.tar.gz \
    taxdump.tar.gz.md5 \
    assembly_summary_genbank.txt \
    assembly_summary_refseq.txt

rm \
    *.dmp \
    taxdump.tar.gz \
    taxdump.tar.gz.md5 \
    assembly_summary_genbank.txt \
    assembly_summary_refseq.txt
```

### Usage of each command

For practical uses of `nwr` and other awesome companions, follow this [page](docs/ncbi_ar.md).

```shell
# nwr download

# nwr txdb

nwr info "Homo sapiens" 4932

nwr lineage "Homo sapiens"
nwr lineage 4932

nwr restrict "Vertebrata" -c 2 -f tests/nwr/taxon.tsv
##sci_name       tax_id
#Human   9606

nwr member "Homo"

nwr append tests/nwr/taxon.tsv -c 2 -r species -r family --id

# nwr ardb
# nwr ardb --genbank

nwr common "Escherichia coli" 4932 Drosophila_melanogaster 9606 Mus_musculus
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


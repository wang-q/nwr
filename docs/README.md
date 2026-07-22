# Introduction

`nwr` is a command line **N**CBI taxonomy and assembly **WR**angler. It provides
tools to manage species taxonomy information and process genome assembly
metadata from NCBI.

## Installation

See the [project README](https://github.com/wang-q/nwr#install) for installation
options (`cargo install nwr`, pre-compiled binaries, or `cbp install nwr`).

## Quick Start

```shell
# Download and build the local databases
nwr download
nwr txdb
nwr ardb

# Query taxonomy
nwr info "Homo sapiens"
nwr lineage 9606
nwr member "Homo" -r species
```

## Documentation

- **User Guide** — practical recipes with NCBI assembly reports.
- **Command Reference** — detailed behavior and examples for every subcommand.
- **Database Schema** — tables and columns of the SQLite databases.

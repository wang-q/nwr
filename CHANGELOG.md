# Change Log

## Unreleased - ReleaseDate

* Fix multiple rounds of condensing leaf nodes
* Skip invalid lines
    * append
    * ardb
    * pl-condense

## 0.7.0 - 2023-09-15

* Add `nwr pl-condense`

* Add `--column` to `nwr label`
* Add `-I` and `-L` to `nwr replace`

* Fix `--term` in `nwr subtree`
* Handle more tags in `nwr tex`

## 0.6.5 - 2023-09-12

* Add `nwr distance`
* Add `nwr replace`
* Add `nwr prune`
* Add `nwr reroot`

* Enhance `nwr label`
    * Various filtering methods
    * `--descendants` - Internal nodes' descendants will automatically be included
* Enhance `nwr subtree`
    * `--condense` - Condense the subtree with the name provided

* Adjust `template.tex` and update trees

## 0.6.4 - 2023-09-09

* Add `nwr common`
* Add `nwr subtree`
* Add `nwr topo`

## 0.6.3 - 2023-09-07

* Add `nwr indent`
* Add `nwr order`
* Add `nwr label`
* Add `nwr rename`
* Add `nwr comment`
* Add `nwr stat`
* Add `nwr tex`

* Add a directory `tree/` to store phylogenetic trees

## 0.6.2 - 2023-07-18

* Add more templates to `nwr template`
    * `--mh` for MinHash
    * `--count` for counting
    * `--pro` for collecting proteins
    * `--in` and `--not-in` to including and excluding assemblies in some steps

* Add `nwr kb` to bundle HMM files
    * `bac120`
    * `ar53`
    * `fungi61`

## 0.6.0 - 2023-06-03

* Add `nwr template` to replace `nwr assembly` and temporally existed `nwr biosample`
    * `nwr template --ass`
    * `nwr template --bs`
* Add columns `infraspecific_name` and `gbrs_paired_asm` to ardb

## 0.5.10 - 2023-01-28

* Add `nwr assembly`
* Add `-e` to `nwr restrict`
* Add database schema

## 0.5.9 - 2022-12-13

* Bump versions of deps
    * clap v4
    * Use anyhow

## 0.5.7 - 2022-04-10

* Update the CI workflows
    * Remove travis and appveyor
    * Use a container with GLIBC 2.17 according to
      this [blog post](https://kobzol.github.io/rust/ci/2021/05/07/building-rust-binaries-in-ci-that-work-with-older-glibc.html)
* Remove old scripts

## 0.5.5 - 2022-03-04

* Check organism_name with the one in txdb
* Add column biosample to ardb

## 0.5.4 - 2022-03-02

* Add `nwr ardb`

## 0.5.3 - 2022-02-24

* Move old Perl codes here
* Add `doc/assembly.md`

## 0.5.0 - 2022-02-22

* Extract `nwr` to a standalone repo.
    * `SQLite` can't be built statically under musl

## 0.4.17 - 2022-02-21

* Add `nwr download`
* Add `nwr txdb`
* Add `nwr info`
* Add `nwr lineage`
* Add `nwr restrict`
* Add `nwr member`
* Add `nwr append`
* Use `taxdump.tar.gz` instead of `taxdmp.zip` to avoid the `zip` crate

## 0.4.16 - 2022-02-12

* Switch to `clap` v3
* New binary `nwr`

# Change Log

## Unreleased - ReleaseDate

## 0.5.7 - 2022-04-10

## 0.5.6 - 2022-04-09

* Update the CI workflows
    * Remove travis and appveyor
    * Use a container with GLIBC 2.17 according to
      this [blog](https://kobzol.github.io/rust/ci/2021/05/07/building-rust-binaries-in-ci-that-work-with-older-glibc.html)
      post
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

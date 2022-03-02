# Change Log

## Unreleased - ReleaseDate

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

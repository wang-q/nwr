/// Abbreviation generation for strain/species/genus names.
pub mod abbr;
/// Append taxonomic rank columns to TSV files.
pub mod append;
/// Assembly report database builder.
pub mod ardb;
/// Shared helpers for taxonomy tree operations.
pub mod common;
/// NCBI taxonomy and assembly report downloader.
pub mod download;
/// List taxonomy members under ancestor terms.
pub mod member;
/// Restrict TSV lines to descendants of ancestor terms.
pub mod restrict;
/// Sequence metadata database builder.
pub mod seqdb;
/// NCBI taxonomy database queries and helpers.
pub mod taxonomy;
/// Phylogenomic pipeline template generator.
pub mod template;
/// Taxonomy database builder.
pub mod txdb;

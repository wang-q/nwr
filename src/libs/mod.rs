/// Abbreviation generation for strain/species/genus names.
pub mod abbr;
/// NCBI taxonomy and assembly report downloader.
pub mod download;
/// I/O helpers returning `Result` instead of panicking.
pub mod io;
/// Sequence metadata database builder.
pub mod seqdb;
/// NCBI taxonomy queries and operations.
pub mod taxonomy;
/// Phylogenomic pipeline template generator.
pub mod template;

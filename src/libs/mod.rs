// Skip repetitive `# Errors` doc sections for every thin I/O/database Result
// wrapper; the project convention is a one-line doc comment for public items.
#![allow(clippy::missing_errors_doc)]

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

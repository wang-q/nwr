// `execute` functions are intentionally long because they orchestrate the
// whole subcommand in one place; skip repetitive `# Errors` doc sections.
#![allow(clippy::missing_errors_doc, clippy::too_many_lines)]

//! Subcommand modules for the `nwr` binary.

/// Generate abbreviated strain names.
pub mod abbr;
/// Append taxonomy columns to input files.
pub mod append;
/// Build the assembly report database.
pub mod ardb;
/// Shared clap argument definitions.
pub mod args;
/// Find common ancestors of taxa.
pub mod common;
/// Download NCBI taxonomy and assembly reports.
pub mod download;
/// Display taxonomy information for terms.
pub mod info;
/// Print a knowledge base TSV for assemblies.
pub mod kb;
/// Output taxonomic lineages.
pub mod lineage;
/// List members of a taxonomic group.
pub mod member;
/// Include or exclude rows by taxonomy.
pub mod restrict;
/// Build and populate the sequence metadata database.
pub mod seqdb;
/// Generate phylogenomic pipeline templates.
pub mod template;
/// Build the NCBI taxonomy database.
pub mod txdb;

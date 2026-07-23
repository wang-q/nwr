// `execute` functions are intentionally long because they orchestrate the
// whole subcommand in one place; skip repetitive `# Errors` doc sections.
#![allow(clippy::missing_errors_doc, clippy::too_many_lines)]

//! Subcommand modules for the `nwr` binary.

pub mod abbr;
pub mod append;
pub mod ardb;
pub mod args;
pub mod common;
pub mod download;
pub mod info;
pub mod kb;
pub mod lineage;
pub mod member;
pub mod restrict;
pub mod seqdb;
pub mod template;
pub mod txdb;

//! `nwr` - NCBI taxonomy and assembly wrangler.
//!
//! This crate provides the shared library surface used by the `nwr` CLI.
//! Subcommand implementations are split into [`libs`] modules; thin CLI
//! handlers live in [`cmd_nwr`](crate) (binary-only).

/// Internal shared libraries used by `nwr` subcommands.
pub mod libs;

pub use crate::libs::taxonomy::*;

//! Shared `clap::Arg` builders reused across subcommands.
//!
//! Keeps the option name, short flag, help text and default value consistent
//! for arguments that appear in more than one subcommand, as required by
//! `AGENTS.md`.

use clap::{Arg, ArgAction};

/// `--dir` (`-d`) option pointing at the NWR data directory.
#[must_use]
pub fn dir_arg() -> Arg {
    Arg::new("dir")
        .long("dir")
        .short('d')
        .num_args(1)
        .value_name("DIR")
        .help("Specify the NWR data directory")
}

/// `--outfile` (`-o`) option for output file path (defaults to stdout).
#[must_use]
pub fn outfile_arg() -> Arg {
    Arg::new("outfile")
        .short('o')
        .long("outfile")
        .num_args(1)
        .default_value("stdout")
        .help("Output filename (default: stdout)")
}

/// `--rank` (`-r`) option for taxonomic rank(s), repeatable.
#[must_use]
pub fn rank_arg() -> Arg {
    Arg::new("rank")
        .long("rank")
        .short('r')
        .num_args(1..)
        .action(ArgAction::Append)
        .help("Taxonomic rank(s)")
}

/// `--column` (`-c`) option for 1-based column index (defaults to 1).
/// Rejects 0 at CLI parse time so users get an immediate, targeted error.
#[must_use]
pub fn column_arg() -> Arg {
    Arg::new("column")
        .long("column")
        .short('c')
        .num_args(1)
        .default_value("1")
        .value_parser(clap::builder::RangedU64ValueParser::<usize>::new().range(1..))
        .help("Column number (1-based)")
}

/// `--outdir` option for output directory (defaults to current directory).
#[must_use]
pub fn outdir_arg() -> Arg {
    Arg::new("outdir")
        .long("outdir")
        .num_args(1)
        .default_value(".")
        .help("Output directory (default: current directory)")
}

/// Positional `terms` argument: required, multi-value, index 1.
#[must_use]
pub fn terms_arg(help: &'static str) -> Arg {
    Arg::new("terms")
        .help(help)
        .required(true)
        .num_args(1..)
        .index(1)
}

/// Positional `infiles` argument: required, multi-value, index 1.
#[must_use]
pub fn infiles_arg(help: &'static str) -> Arg {
    Arg::new("infiles")
        .help(help)
        .required(true)
        .num_args(1..)
        .index(1)
}

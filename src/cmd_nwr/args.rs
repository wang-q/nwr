//! Shared `clap::Arg` builders reused across subcommands.
//!
//! Keeps the option name, short flag, help text and default value consistent
//! for arguments that appear in more than one subcommand, as required by
//! `AGENTS.md`.

use clap::{value_parser, Arg, ArgAction};

/// `--dir` (`-d`) option pointing at the NWR data directory.
pub fn dir_arg() -> Arg {
    Arg::new("dir")
        .long("dir")
        .short('d')
        .num_args(1)
        .value_name("DIR")
        .help("Specify the NWR data directory")
}

/// `--outfile` (`-o`) option for output file path (defaults to stdout).
pub fn outfile_arg() -> Arg {
    Arg::new("outfile")
        .short('o')
        .long("outfile")
        .num_args(1)
        .default_value("stdout")
        .help("Output filename (default: stdout)")
}

/// `--rank` (`-r`) option for taxonomic rank(s), repeatable.
pub fn rank_arg() -> Arg {
    Arg::new("rank")
        .long("rank")
        .short('r')
        .num_args(1..)
        .action(ArgAction::Append)
        .help("Taxonomic rank(s)")
}

/// `--column` (`-c`) option for 1-based column index (defaults to 1).
pub fn column_arg() -> Arg {
    Arg::new("column")
        .long("column")
        .short('c')
        .num_args(1)
        .default_value("1")
        .value_parser(value_parser!(usize))
        .help("Column number (1-based)")
}

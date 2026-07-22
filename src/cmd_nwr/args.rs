//! Shared `clap::Arg` builders reused across subcommands.
//!
//! Keeps the option name, short flag, help text and default value consistent
//! for arguments that appear in more than one subcommand, as required by
//! `AGENTS.md`.

use clap::Arg;

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

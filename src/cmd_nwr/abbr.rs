use super::args;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use std::collections::HashSet;
use std::io::BufRead;
use std::io::Write;

/// Create clap subcommand arguments
#[must_use]
pub fn make_subcommand() -> Command {
    Command::new("abbr")
        .about("Abbreviates strain scientific names")
        .after_help(include_str!("../../docs/help/abbr.md"))
        .arg(
            Arg::new("infile")
                .help("Input file to process. Use 'stdin' for standard input")
                .num_args(1)
                .default_value("stdin")
                .index(1),
        )
        .arg(
            Arg::new("columns")
                .long("columns")
                .short('C')
                .num_args(1)
                .default_value("1,2,3")
                .help("Columns of strain,species,genus (1-based)"),
        )
        .arg(
            Arg::new("separator")
                .long("separator")
                .short('s')
                .num_args(1)
                .default_value("\t")
                .help("Field separator"),
        )
        .arg(
            Arg::new("min")
                .long("min")
                .short('m')
                .num_args(1)
                .default_value("3")
                .value_parser(value_parser!(usize))
                .help("Minimal length for species abbreviation"),
        )
        .arg(
            Arg::new("tight")
                .long("tight")
                .action(ArgAction::SetTrue)
                .help("No underscore between genus and species"),
        )
        .arg(
            Arg::new("shortsub")
                .long("shortsub")
                .action(ArgAction::SetTrue)
                .help("Clean subspecies parts"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let column_str = args
        .get_one::<String>("columns")
        .ok_or_else(|| anyhow::anyhow!("Missing 'columns' argument"))?;
    let cols: Vec<usize> = column_str
        .split(',')
        .map(|s| {
            s.parse()
                .map_err(|_| anyhow::anyhow!("Invalid column number: '{s}'"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    if cols.len() != 3 {
        return Err(anyhow::anyhow!(
            "Column must be in format 's,p,g' (three numbers)"
        ));
    }
    for (i, col) in cols.iter().enumerate() {
        if *col == 0 {
            return Err(anyhow::anyhow!(
                "Column {} must be a positive integer (1-based)",
                i + 1
            ));
        }
    }

    let columns = (cols[0], cols[1], cols[2]);
    let separator = args
        .get_one::<String>("separator")
        .ok_or_else(|| anyhow::anyhow!("Missing 'separator' argument"))?;
    let min_len = *args
        .get_one::<usize>("min")
        .ok_or_else(|| anyhow::anyhow!("Missing 'min' argument"))?;
    let tight = args.get_flag("tight");
    let shortsub = args.get_flag("shortsub");

    let reader = nwr::libs::io::reader(
        args.get_one::<String>("infile")
            .ok_or_else(|| anyhow::anyhow!("Missing 'infile' argument"))?,
    )?;
    let mut writer = nwr::libs::io::writer(
        args.get_one::<String>("outfile")
            .ok_or_else(|| anyhow::anyhow!("Missing 'outfile' argument"))?,
    )?;

    let mut all_fields: Vec<Vec<String>> = Vec::new();
    let mut all_parts: Vec<nwr::libs::abbr::NameParts> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        match nwr::libs::abbr::process_line(&line, columns, separator, shortsub) {
            Some((fields, parts)) => {
                all_fields.push(fields);
                all_parts.push(parts);
            }
            None => {
                eprintln!("Warning: skipping malformed line: {line}");
            }
        }
    }

    let genus_list: Vec<String> = all_parts
        .iter()
        .filter(|p| p.is_normal)
        .map(|p| p.genus.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let species_list: Vec<String> = all_parts
        .iter()
        .filter(|p| p.is_normal)
        .map(|p| p.species.clone())
        .filter(|s| !s.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let genus_abbr = nwr::libs::abbr::abbr_most(&genus_list, 1, true);
    let species_abbr = nwr::libs::abbr::abbr_most(&species_list, min_len, true);

    for (i, parts) in all_parts.iter().enumerate() {
        let fields = &all_fields[i];
        let original_line = fields.join(separator);

        let abbr = if parts.is_normal {
            let spacer = if tight { "" } else { "_" };
            let ge = genus_abbr.get(&parts.genus).unwrap_or(&parts.genus);
            let sp = species_abbr.get(&parts.species).unwrap_or(&parts.species);

            let ge_sp = if parts.species.is_empty() {
                ge.clone()
            } else {
                format!("{ge}{spacer}{sp}")
            };

            if parts.strain.is_empty() {
                ge_sp
            } else {
                format!("{}_{}", ge_sp, parts.strain)
            }
        } else {
            parts.strain.clone()
        };

        writer.write_fmt(format_args!("{original_line}\t{abbr}\n"))?;
    }
    writer.flush()?;

    Ok(())
}

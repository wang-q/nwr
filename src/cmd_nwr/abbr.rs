use clap::*;
use std::collections::HashSet;
use std::io::BufRead;

/// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("abbr")
        .about("Abbreviate strain scientific names")
        .after_help(include_str!("../../docs/help/abbr.md"))
        .arg(
            Arg::new("infile")
                .help("Input file to process. Use 'stdin' for standard input")
                .num_args(1)
                .default_value("stdin")
                .index(1),
        )
        .arg(
            Arg::new("column")
                .long("column")
                .short('c')
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
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename (default: stdout)"),
        )
}

/// Command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let outfile = args.get_one::<String>("outfile").unwrap();
    let column_str = args.get_one::<String>("column").unwrap();
    let separator = args.get_one::<String>("separator").unwrap();
    let min_len: usize = *args.get_one("min").unwrap();
    let tight = args.get_flag("tight");
    let shortsub = args.get_flag("shortsub");

    // Parse columns
    let cols: Vec<usize> = column_str
        .split(',')
        .map(|s| {
            s.parse()
                .map_err(|_| anyhow::anyhow!("Invalid column number: '{}'", s))
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

    // Read all lines
    let reader = intspan::reader(infile);
    let mut all_fields: Vec<Vec<String>> = Vec::new();
    let mut all_parts: Vec<nwr::libs::abbr::NameParts> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((fields, parts)) =
            nwr::libs::abbr::process_line(&line, columns, separator, shortsub)
        {
            all_fields.push(fields);
            all_parts.push(parts);
        }
    }

    // Collect unique genus and species for normal entries
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

    // Generate abbreviations
    let genus_abbr = nwr::libs::abbr::abbr_most(&genus_list, 1, true);
    let species_abbr = nwr::libs::abbr::abbr_most(&species_list, min_len, true);

    // Output results
    let mut writer = intspan::writer(outfile);

    for (i, parts) in all_parts.iter().enumerate() {
        let fields = &all_fields[i];
        let original_line = fields.join(separator);

        let abbr = if parts.is_normal {
            let spacer = if tight { "" } else { "_" };
            let ge = genus_abbr.get(&parts.genus).unwrap_or(&parts.genus);
            let sp = species_abbr.get(&parts.species).unwrap_or(&parts.species);

            let ge_sp = if parts.species.is_empty() {
                ge.to_string()
            } else {
                format!("{}{}{}", ge, spacer, sp)
            };

            if parts.strain.is_empty() {
                ge_sp
            } else {
                format!("{}_{}", ge_sp, parts.strain)
            }
        } else {
            parts.strain.clone()
        };

        writer.write_fmt(format_args!("{}\t{}\n", original_line, abbr))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbr_with_zero_column() {
        let cmd = make_subcommand();
        let matches = cmd
            .try_get_matches_from(["abbr", "tests/nwr/strains.tsv", "--column", "0,2,3"])
            .unwrap();

        let result = execute(&matches);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("positive integer"));
    }
}

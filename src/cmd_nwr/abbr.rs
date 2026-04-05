use clap::*;
use regex::Regex;
use std::collections::HashMap;
use std::io::BufRead;

/// Structure to hold name parts
struct NameParts {
    strain: String,
    species: String,
    genus: String,
    is_normal: bool,
}

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

/// Generate unique abbreviations for a list of words (similar to Perl's Text::Abbrev).
///
/// For each word, generates all possible abbreviations from `min_len` to the full word length.
/// An abbreviation is valid only if it uniquely identifies a single word.
///
/// # Arguments
/// * `words` - List of words to abbreviate
/// * `min_len` - Minimum length for abbreviations
///
/// # Returns
/// A HashMap mapping valid abbreviations to their full words
fn abbr(words: &[String], min_len: usize) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut table: HashMap<String, usize> = HashMap::new();

    for word in words {
        let word_len = word.len();
        for len in (min_len..word_len).rev() {
            let abbrev = &word[..len];
            let seen = table.entry(abbrev.to_string()).or_insert(0);
            *seen += 1;

            if *seen == 1 {
                // First word with this abbreviation
                result.insert(abbrev.to_string(), word.clone());
            } else if *seen == 2 {
                // Second word - can't use this abbreviation
                result.remove(abbrev);
            }
            // Third or more - skip
        }
    }

    // Non-abbreviations always get entered
    for word in words {
        result.insert(word.clone(), word.clone());
    }

    result
}

/// Select the longest valid abbreviation for each word.
///
/// Builds on `abbr()` to find the longest unique abbreviation for each word.
/// When `creat` is true, avoids abbreviating words that differ by only one character.
///
/// # Arguments
/// * `words` - List of words to abbreviate
/// * `min_len` - Minimum length for abbreviations
/// * `creat` - If true, don't abbreviate when only 1 character would be saved
///
/// # Returns
/// A HashMap mapping each full word to its longest valid abbreviation
fn abbr_most(words: &[String], min_len: usize, creat: bool) -> HashMap<String, String> {
    if words.is_empty() {
        return HashMap::new();
    }

    // Don't abbreviate if min_len is 0
    if min_len == 0 {
        return words.iter().map(|w| (w.clone(), w.clone())).collect();
    }

    let abbr_map = abbr(words, min_len);
    let mut sorted_keys: Vec<&String> = abbr_map.keys().collect();
    sorted_keys.sort();

    let mut abbr_of: HashMap<String, String> = HashMap::new();

    for i in (1..sorted_keys.len()).rev() {
        let key = sorted_keys[i];
        let prev_key = sorted_keys[i - 1];

        if !key.starts_with(prev_key) {
            if let Some(full) = abbr_map.get(key) {
                abbr_of.insert(full.clone(), key.clone());
            }
        }
    }

    // Handle the first key
    if let Some(first_key) = sorted_keys.first() {
        if let Some(full) = abbr_map.get(*first_key) {
            if !abbr_of.contains_key(full) {
                abbr_of.insert(full.clone(), (*first_key).clone());
            }
        }
    }

    // Don't abbreviate 1 letter difference
    if creat {
        let keys_to_update: Vec<(String, String)> = abbr_of
            .iter()
            .filter(|(k, v)| k.len() - v.len() == 1)
            .map(|(k, _v)| (k.clone(), k.clone()))
            .collect();
        for (k, v) in keys_to_update {
            abbr_of.insert(k, v);
        }
    }

    abbr_of
}

/// Clean name by replacing non-alphanumeric characters with underscores.
///
/// Removes leading and trailing underscores, and collapses consecutive
/// underscores into a single one.
///
/// # Arguments
/// * `name` - The name to clean
///
/// # Returns
/// The cleaned name containing only alphanumeric characters and single underscores
fn clean_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    cleaned.replace("__", "_").trim_matches('_').to_string()
}

/// Clean subspecies parts using word boundary regex (equivalent to Perl \b).
///
/// Removes common subspecies designation terms like "subsp", "strain", "serovar",
/// etc. from strain names to produce cleaner abbreviations.
///
/// # Arguments
/// * `strain` - The strain name to clean
///
/// # Returns
/// The strain name with subspecies designations removed
fn clean_subspecies(strain: &str) -> String {
    let patterns = [
        "subsp",
        "serovar",
        "str",
        "strain",
        "substr",
        "serotype",
        "biovar",
        "var",
        "group",
        "variant",
        "genomovar",
        "genomosp",
        "breed",
        "cultivar",
        "ecotype",
        "n/a",
        "NA",
        "microbial",
        "clinical",
        "pathogenic",
        "isolate",
    ];

    let mut result = strain.to_string();
    for pattern in &patterns {
        // Create regex with word boundaries, case-insensitive
        let regex_str = format!(r"(?i)\b{}\b", regex::escape(pattern));
        if let Ok(re) = Regex::new(&regex_str) {
            result = re.replace_all(&result, "").to_string();
        }
    }
    result
}

/// Process a single line and extract name parts for abbreviation.
///
/// Parses a line using the specified separator and column indices to extract
/// strain, species, and genus information.
///
/// # Arguments
/// * `line` - The input line to process
/// * `columns` - Tuple of (strain_col, species_col, genus_col) as 1-based indices
/// * `separator` - Field separator string
/// * `shortsub` - Whether to clean subspecies parts
///
/// # Returns
/// Option containing the original fields and extracted NameParts
fn process_line(
    line: &str,
    columns: (usize, usize, usize),
    separator: &str,
    shortsub: bool,
) -> Option<(Vec<String>, NameParts)> {
    if line.is_empty() {
        return None;
    }

    let fields: Vec<String> = line.split(separator).map(|s| s.to_string()).collect();
    if fields.len() < columns.2 {
        return None;
    }

    let strain = fields.get(columns.0 - 1)?.trim().replace(['"', '\''], "");
    let species = fields.get(columns.1 - 1)?.trim().replace(['"', '\''], "");
    let genus = fields.get(columns.2 - 1)?.trim().replace(['"', '\''], "");

    let mut is_normal = false;
    let mut strain_clean = strain.clone();
    let mut species_clean = species.clone();
    let mut genus_clean = genus.clone();

    if genus != species {
        // Normal case: genus starts with word char and species starts with genus
        if genus.chars().next()?.is_alphabetic() {
            if species.starts_with(&genus) {
                if strain.starts_with(&species) {
                    strain_clean =
                        strain.trim_start_matches(&species).trim_start().to_string();
                    species_clean =
                        species.trim_start_matches(&genus).trim_start().to_string();
                    is_normal = true;
                }
            }
        }
    } else {
        // No species part
        if genus.chars().next()?.is_alphabetic() {
            if strain.starts_with(&genus) {
                strain_clean =
                    strain.trim_start_matches(&genus).trim_start().to_string();
                species_clean = String::new();
                is_normal = true;
            }
        }
    }

    // Remove Candidatus
    genus_clean = genus_clean.replace("Candidatus ", "C");
    genus_clean = genus_clean.replace("candidatus ", "C");
    genus_clean = genus_clean.replace("CANDIDATUS ", "C");

    // Clean subspecies if requested
    if shortsub {
        strain_clean = clean_subspecies(&strain_clean);
    }

    // Clean names
    strain_clean = clean_name(&strain_clean);
    species_clean = clean_name(&species_clean);
    genus_clean = clean_name(&genus_clean);

    Some((
        fields,
        NameParts {
            strain: strain_clean,
            species: species_clean,
            genus: genus_clean,
            is_normal,
        },
    ))
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
    let columns = (cols[0], cols[1], cols[2]);

    // Read all lines
    let reader = intspan::reader(infile);
    let mut all_fields: Vec<Vec<String>> = Vec::new();
    let mut all_parts: Vec<NameParts> = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        if let Some((fields, parts)) = process_line(&line, columns, separator, shortsub)
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
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let species_list: Vec<String> = all_parts
        .iter()
        .filter(|p| p.is_normal)
        .map(|p| p.species.clone())
        .filter(|s| !s.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Generate abbreviations
    let genus_abbr = abbr_most(&genus_list, 1, true);
    let species_abbr = abbr_most(&species_list, min_len, true);

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

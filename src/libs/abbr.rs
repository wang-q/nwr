use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Common subspecies designation terms removed by [`clean_subspecies`].
pub const SUBSPECIES_TERMS: &[&str] = &[
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

static SUBSPECIES_REGEXS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?xi)\b({})\b",
        SUBSPECIES_TERMS
            .iter()
            .map(|t| regex::escape(t))
            .collect::<Vec<_>>()
            .join("|")
    ))
    .unwrap()
});

/// Matches the "Candidatus" prefix case-insensitively for abbreviation.
static RE_CANDIDATUS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Candidatus ").unwrap());

/// Parsed name parts extracted from an input line.
pub struct NameParts {
    /// Strain name (the full input name with subspecies terms removed).
    pub strain: String,
    /// Species-level scientific name.
    pub species: String,
    /// Genus name.
    pub genus: String,
    /// Whether the name follows the expected `genus species strain` pattern.
    pub is_normal: bool,
}

/// Generate unique abbreviations for a list of words (similar to Perl's `Text::Abbrev`).
///
/// For each word, generates all possible abbreviations from `min_len` to the full word length.
/// An abbreviation is valid only if it uniquely identifies a single word.
#[must_use]
pub fn abbr(words: &[String], min_len: usize) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut table: HashMap<String, usize> = HashMap::new();

    for word in words {
        let chars: Vec<char> = word.chars().collect();
        let word_len = chars.len();
        for len in (min_len..word_len).rev() {
            let abbrev: String = chars[..len].iter().collect();
            let seen = table.entry(abbrev.clone()).or_insert(0);
            *seen += 1;

            if *seen == 1 {
                // We're the first word so far to have this abbreviation
                result.insert(abbrev, word.clone());
            } else if *seen == 2 {
                // We're the second word to have this abbreviation,
                // so we can't use it
                result.remove(&abbrev);
            }
            // We're the third word to have this abbreviation,
            // so skip to the next word
        }
    }

    // Non-abbreviations always get entered, even if they aren't unique
    for word in words {
        result.insert(word.clone(), word.clone());
    }

    result
}

/// Select the shortest unique prefix abbreviation for each word.
///
/// Builds on [`abbr`] to find the shortest prefix that uniquely identifies
/// each word. When `avoid_one_char_saving` is true, avoids abbreviating words
/// that differ by only one character.
#[must_use]
pub fn abbr_most(
    words: &[String],
    min_len: usize,
    avoid_one_char_saving: bool,
) -> HashMap<String, String> {
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

        // Skip only when this key is a longer abbreviation of the *same* word
        // as the previous key. Without the value check, distinct words that
        // share a prefix (e.g. "app" and "apple") would lose the longer word
        // entirely.
        let same_word = abbr_map.get(key) == abbr_map.get(prev_key);
        if !key.starts_with(prev_key) || !same_word {
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
    if avoid_one_char_saving {
        let keys_to_update: Vec<(String, String)> = abbr_of
            .iter()
            .filter(|(k, v)| k.chars().count() - v.chars().count() == 1)
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
#[must_use]
pub fn clean_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    // Collapse consecutive underscores into a single one and trim edges
    cleaned
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Clean subspecies parts using word boundary regex (equivalent to Perl `\b`).
///
/// Removes common subspecies designation terms like "subsp", "strain", "serovar",
/// etc. from strain names to produce cleaner abbreviations.
#[must_use]
pub fn clean_subspecies(strain: &str) -> String {
    SUBSPECIES_REGEXS.replace_all(strain, "").to_string()
}

/// Process a single line and extract name parts for abbreviation.
///
/// Parses a line using the specified separator and column indices to extract
/// strain, species, and genus information.
#[must_use]
pub fn process_line(
    line: &str,
    columns: (usize, usize, usize),
    separator: &str,
    shortsub: bool,
) -> Option<(Vec<String>, NameParts)> {
    if line.is_empty() {
        return None;
    }
    // Columns are 1-based; reject 0 to avoid usize underflow on `columns.n - 1`.
    if columns.0 == 0 || columns.1 == 0 || columns.2 == 0 {
        return None;
    }

    let fields: Vec<String> = line
        .split(separator)
        .map(std::string::ToString::to_string)
        .collect();
    // Require enough fields for the largest requested column index; columns
    // may be in any order (e.g. `--columns 3,2,1`), so checking only
    // `columns.2` would under-validate.
    let max_col = columns.0.max(columns.1).max(columns.2);
    if fields.len() < max_col {
        return None;
    }

    let strain = fields.get(columns.0 - 1)?.trim().replace(['"', '\''], "");
    let species = fields.get(columns.1 - 1)?.trim().replace(['"', '\''], "");
    let genus = fields.get(columns.2 - 1)?.trim().replace(['"', '\''], "");

    let mut is_normal = false;
    let mut strain_clean = strain.clone();
    let mut species_clean = species.clone();
    let mut genus_clean = genus.clone();

    let genus_starts_alpha = genus.chars().next().is_some_and(char::is_alphabetic);

    if genus != species
        && genus_starts_alpha
        && species.starts_with(&genus)
        && strain.starts_with(&species)
    {
        // Normal case: genus starts with word char and species starts with genus.
        // strip_prefix removes only the first occurrence; trim_start_matches
        // would incorrectly remove all leading repetitions.
        strain_clean = strain
            .strip_prefix(&species)
            .unwrap_or(&strain[..])
            .trim_start()
            .to_string();
        species_clean = species
            .strip_prefix(&genus)
            .unwrap_or(&species[..])
            .trim_start()
            .to_string();
        is_normal = true;
    } else if genus == species && genus_starts_alpha && strain.starts_with(&genus) {
        // No species part
        strain_clean = strain
            .strip_prefix(&genus)
            .unwrap_or(&strain[..])
            .trim_start()
            .to_string();
        species_clean = String::new();
        is_normal = true;
    }

    // Remove Candidatus (case-insensitive)
    genus_clean = RE_CANDIDATUS.replace_all(&genus_clean, "C").to_string();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_name_with_special_chars() {
        assert_eq!(clean_name("a/b@c"), "a_b_c");
        assert_eq!(clean_name("a__b"), "a_b");
        assert_eq!(clean_name("_abc_"), "abc");
    }

    #[test]
    fn test_clean_subspecies_basic() {
        assert_eq!(clean_subspecies("strain ABC"), " ABC");
        assert_eq!(clean_subspecies("subsp. XYZ"), ". XYZ");
    }

    #[test]
    fn test_abbr_basic() {
        let words = vec!["apple".to_string(), "apricot".to_string()];
        let result = abbr(&words, 1);
        assert_eq!(result.get("apple").unwrap(), "apple");
        assert_eq!(result.get("apricot").unwrap(), "apricot");
    }

    #[test]
    fn test_abbr_most_basic() {
        let words = vec!["apple".to_string(), "apricot".to_string()];
        let result = abbr_most(&words, 1, false);
        assert_eq!(result.get("apple").unwrap(), "app");
        assert_eq!(result.get("apricot").unwrap(), "apr");
    }

    #[test]
    fn test_abbr_most_empty() {
        let result = abbr_most(&[], 1, false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_abbr_most_min_len_zero() {
        let words = vec!["apple".to_string()];
        let result = abbr_most(&words, 0, false);
        assert_eq!(result.get("apple").unwrap(), "apple");
    }

    #[test]
    fn test_abbr_most_prefix_words() {
        // "app" is a prefix of "apple"; both must appear in the result.
        let words = vec!["app".to_string(), "apple".to_string()];
        let result = abbr_most(&words, 1, false);
        assert_eq!(result.get("app").unwrap(), "app");
        assert_eq!(result.get("apple").unwrap(), "appl");
    }

    #[test]
    fn test_process_line_normal() {
        let line = "Escherichia coli K-12\tEscherichia coli\tEscherichia";
        let (fields, parts) = process_line(line, (1, 2, 3), "\t", false).unwrap();
        assert_eq!(fields.len(), 3);
        assert!(parts.is_normal);
        assert_eq!(parts.genus, "Escherichia");
        assert_eq!(parts.species, "coli");
    }

    #[test]
    fn test_process_line_empty() {
        assert!(process_line("", (1, 2, 3), "\t", false).is_none());
    }

    #[test]
    fn test_process_line_candidatus() {
        let line = "Candidatus Foo bar strain X\tCandidatus Foo bar\tCandidatus Foo";
        let (_, parts) = process_line(line, (1, 2, 3), "\t", false).unwrap();
        assert!(parts.is_normal);
        assert_eq!(parts.genus, "CFoo");
        assert_eq!(parts.species, "bar");
        assert_eq!(parts.strain, "strain_X");
    }
}

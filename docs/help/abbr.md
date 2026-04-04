# abbr

Behavior:

* Abbreviates strain scientific names to unique short identifiers.
* Generates abbreviations for genus, species, and strain parts.
* Handles special cases like Candidatus and subspecies names.
* Ensures uniqueness of abbreviations across all input names.

Input:

* Accepts a TSV/CSV file or standard input.
* Each row should contain strain, species, and genus names in separate columns.
* Use `--column` to specify which columns contain these names (default: 1,2,3).
* Common column patterns:
  * `1,2,3` - strain in column 1, species in 2, genus in 3
  * `1,1,2` - no strain: strain and species both in column 1, genus in 2
  * `2,2,3` - don't need strain part: strain and species in 2, genus in 3
  * `1,1,1` - only strain: all three in column 1

Output:

* Original line followed by a tab and the generated abbreviation.
* Abbreviation format:
  * Normal mode: `Genus_Species_Strain` (e.g., H_sapiens_sapiens)
  * Tight mode (`--tight`): `GenusSpecies_Strain` (e.g., Hsapiens_sapiens)
* Special handling:
  * Candidatus is abbreviated to C
  * Non-alphanumeric characters are replaced with underscores
  * Consecutive underscores are collapsed
  * Leading and trailing underscores are removed

Examples:

1. Basic usage with default columns
   `echo -e 'Homo sapiens,Homo\nHomo erectus,Homo' | nwr abbr -s ',' -c "1,1,2"`

2. Tight mode (no underscore between genus and species)
   `echo -e 'Homo sapiens,Homo\nHomo erectus,Homo' | nwr abbr -s ',' -c "1,1,2" --tight`

3. Clean subspecies names
   `echo 'Legionella pneumophila subsp. pneumophila' | nwr abbr --shortsub`

4. Process a file
   `nwr abbr names.tsv -o abbreviated.tsv`

5. Custom separator and columns
   `nwr abbr data.csv -s ',' -c "1,2,3" -o output.tsv`

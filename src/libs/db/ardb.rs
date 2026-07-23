use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Organism names matching this regex are considered incompetent and skipped.
    pub static ref RE_INCOMPETENT: Regex =
        Regex::new(r"(?xi)\b(uncultured|unidentified|bacterium|archaeon|metagenome)\b")
            .unwrap();

    /// Organism names matching this regex are considered viral and skipped.
    pub static ref RE_VIRUS: Regex = Regex::new(r"(?xi)(virus|phage)\b").unwrap();
}

/// DDL for the assembly report SQLite database.
pub static DDL_AR: &str = r"
DROP TABLE IF EXISTS ar;

CREATE TABLE ar (
    tax_id             INTEGER,
    organism_name      VARCHAR (200),
    infraspecific_name VARCHAR (200),
    bioproject         VARCHAR (50),
    biosample          VARCHAR (50),
    assembly_accession VARCHAR (50),
    refseq_category    VARCHAR (50),
    assembly_level     VARCHAR (50),
    genome_rep         VARCHAR (50),
    seq_rel_date       DATE,
    asm_name           VARCHAR (200),
    gbrs_paired_asm    VARCHAR (200),
    ftp_path           VARCHAR (200),
    species            VARCHAR (50),
    species_id         INTEGER,
    genus              VARCHAR (50),
    genus_id           INTEGER,
    family             VARCHAR (50),
    family_id          INTEGER
);

";

/// Column indices in NCBI `assembly_summary_refseq.txt` / `assembly_summary_genbank.txt`.
pub const COL_ASSEMBLY_ACCESSION: usize = 0;
pub const COL_BIOPROJECT: usize = 1;
pub const COL_BIOSAMPLE: usize = 2;
pub const COL_REFSEQ_CATEGORY: usize = 4;
pub const COL_TAX_ID: usize = 5;
pub const COL_ORGANISM_NAME: usize = 7;
pub const COL_INFRASPECIFIC_NAME: usize = 8;
pub const COL_ASSEMBLY_LEVEL: usize = 11;
pub const COL_GENOME_REP: usize = 13;
pub const COL_SEQ_REL_DATE: usize = 14;
pub const COL_ASM_NAME: usize = 15;
pub const COL_GBRS_PAIRED_ASM: usize = 17;
pub const COL_FTP_PATH: usize = 19;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_incompetent_patterns() {
        assert!(RE_INCOMPETENT.is_match("uncultured"));
        assert!(RE_INCOMPETENT.is_match("UNIDENTIFIED"));
        assert!(RE_INCOMPETENT.is_match("Bacterium"));
        assert!(RE_INCOMPETENT.is_match("Archaeon"));
        assert!(RE_INCOMPETENT.is_match("Metagenome"));
    }

    #[test]
    fn test_re_virus_patterns() {
        assert!(RE_VIRUS.is_match("virus"));
        assert!(RE_VIRUS.is_match("VIRUS"));
        assert!(RE_VIRUS.is_match("phage"));
        assert!(RE_VIRUS.is_match("PHAGE"));
    }
}

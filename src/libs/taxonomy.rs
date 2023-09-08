use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Taxon {
    pub tax_id: i64,
    pub parent_tax_id: i64,
    pub rank: String,
    pub division: String,
    pub names: HashMap<String, Vec<String>>, // many synonym or common names
    pub comments: Option<String>,
}

impl std::fmt::Display for Taxon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut lines = String::new();

        let sciname = &self.names.get("scientific name").unwrap()[0];
        let l1 = format!("{} - {}\n", sciname, self.rank);
        let l2 = "-".repeat(l1.len() - 1);
        lines.push_str(&l1);
        lines.push_str(&l2);
        lines.push_str(&format!("\nNCBI Taxonomy ID: {}\n", self.tax_id));

        if self.names.contains_key("synonym") {
            lines.push_str("Same as:\n");
            for synonym in self.names.get("synonym").unwrap() {
                lines.push_str(&format!("* {}\n", synonym));
            }
        }

        if self.names.contains_key("genbank common name") {
            let genbank = &self.names.get("genbank common name").unwrap()[0];
            lines.push_str(&format!("Commonly named {}.\n", genbank));
        }

        if self.names.contains_key("common name") {
            lines.push_str("Also known as:\n");
            for name in self.names.get("common name").unwrap() {
                lines.push_str(&format!("* {}\n", name));
            }
        }

        if self.names.contains_key("authority") {
            lines.push_str("First description:\n");
            for authority in self.names.get("authority").unwrap() {
                lines.push_str(&format!("* {}\n", authority));
            }
        }

        lines.push_str(&format!("Part of the {}.\n", self.division));

        if let Some(ref comments) = self.comments {
            lines.push_str(&format!("\nComments: {}", comments));
        }

        writeln!(f, "{}", lines)
    }
}

pub fn find_rank(lineage: &[Taxon], rank: String) -> (i64, String) {
    let mut tax_id: i64 = 0;
    let mut sci_name = "NA".to_string();

    for node in lineage.iter() {
        if node.rank == rank {
            sci_name = node.names.get("scientific name").unwrap()[0].to_string();
            tax_id = node.tax_id;
            break;
        }
    }

    (tax_id, sci_name)
}

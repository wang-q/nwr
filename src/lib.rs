extern crate log;

mod libs;

pub use crate::libs::newick::*;
pub use crate::libs::taxonomy::*;
pub use crate::libs::txdb::*;

pub fn find_rank(lineage: &Vec<Node>, rank: String) -> (i64, String) {
    let mut tax_id: i64 = 0;
    let mut sci_name = "NA".to_string();

    for node in lineage.into_iter() {
        if node.rank == rank {
            sci_name = (&node.names.get("scientific name").unwrap()[0]).to_string();
            tax_id = node.tax_id;
            break;
        }
    }

    (tax_id, sci_name)
}

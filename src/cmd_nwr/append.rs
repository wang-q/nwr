use super::args;
use clap::*;
use log::warn;
use simplelog::*;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::io::Write;

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("append")
        .about("Appends taxonomic rank fields to a TSV file")
        .after_help(include_str!("../../docs/help/append.md"))
        .arg(args::infiles_arg(
            "Input TSV file(s) to process. Use 'stdin' for standard input",
        ))
        .arg(args::dir_arg())
        .arg(args::rank_arg())
        .arg(args::column_arg())
        .arg(
            Arg::new("id")
                .long("id")
                .action(ArgAction::SetTrue)
                .help("Also append taxon IDs for each rank"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let column: usize = *args.get_one("column").unwrap();

    let ranks: Vec<String> = args
        .get_many::<String>("rank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let infiles: Vec<String> = args
        .get_many::<String>("infiles")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    let outfile = args.get_one::<String>("outfile").unwrap();
    let is_id = args.get_flag("id");

    if column == 0 {
        return Err(anyhow::anyhow!(
            "Column must be a positive integer (1-based)"
        ));
    }

    let mut writer = nwr::libs::io::writer(outfile)?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    // Cache repeated lookups so that input files with duplicate terms don't
    // trigger redundant SQL queries. Failed lookups are also cached so that
    // repeated invalid terms/ids skip without re-querying.
    let mut term_cache: HashMap<String, i64> = HashMap::new();
    let mut term_failed: HashSet<String> = HashSet::new();
    let mut lineage_cache: HashMap<i64, Vec<nwr::Taxon>> = HashMap::new();
    let mut lineage_failed: HashSet<i64> = HashSet::new();
    let mut taxon_cache: HashMap<i64, nwr::Taxon> = HashMap::new();
    let mut taxon_failed: HashSet<i64> = HashSet::new();

    for infile in &infiles {
        let reader = nwr::libs::io::reader(infile)?;

        'line: for (line_idx, line) in reader.lines().enumerate() {
            let line = line?;

            // Lines start with "#"
            if line.starts_with('#') {
                let mut fields: Vec<String> =
                    line.split('\t').map(|s| s.to_string()).collect();
                if ranks.is_empty() {
                    fields.push("sci_name".to_string());
                    if is_id {
                        fields.push("tax_id".to_string());
                    }
                } else {
                    for rank in ranks.iter() {
                        fields.push(rank.to_string());
                        if is_id {
                            fields.push(format!("{}_id", rank));
                        }
                    }
                }
                let new_line: String = fields.join("\t");
                writer.write_fmt(format_args!("{}\n", new_line))?;
                continue;
            }

            let mut fields: Vec<String> =
                line.split('\t').map(|s| s.to_string()).collect();
            // Normal lines
            // Check the given field
            let term = fields.get(column - 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "{}:{}: Column {} out of range (line has {} columns)",
                    infile,
                    line_idx + 1,
                    column,
                    fields.len()
                )
            })?;
            if term_failed.contains(term.as_str()) {
                continue 'line;
            }
            let id = match term_cache.get(term.as_str()) {
                Some(&id) => id,
                None => match nwr::term_to_tax_id(&conn, term) {
                    Ok(x) => {
                        term_cache.insert(term.clone(), x);
                        x
                    }
                    Err(err) => {
                        warn!("Error converting term '{}': {}", term, err);
                        term_failed.insert(term.clone());
                        continue 'line;
                    }
                },
            };

            if ranks.is_empty() {
                if taxon_failed.contains(&id) {
                    continue 'line;
                }
                if let Entry::Vacant(e) = taxon_cache.entry(id) {
                    match nwr::get_taxon(&conn, &[id]) {
                        Ok(x) => {
                            let n = x.into_iter().next().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "get_taxon returned no taxa for id {}",
                                    id
                                )
                            })?;
                            e.insert(n);
                        }
                        Err(err) => {
                            warn!("Error getting taxon {}: {}", id, err);
                            taxon_failed.insert(id);
                            continue 'line;
                        }
                    }
                }
                let node = taxon_cache.get(&id).unwrap();
                let s = node.scientific_name().unwrap_or("Unknown").to_string();

                fields.push(s);
                if is_id {
                    fields.push(id.to_string());
                }
            } else {
                if lineage_failed.contains(&id) {
                    continue 'line;
                }
                if let Entry::Vacant(e) = lineage_cache.entry(id) {
                    match nwr::get_lineage(&conn, id) {
                        Err(err) => {
                            warn!("Errors on get_lineage({}): {}", id, err);
                            lineage_failed.insert(id);
                            continue 'line;
                        }
                        Ok(x) => {
                            e.insert(x);
                        }
                    }
                }
                let lineage = lineage_cache.get(&id).unwrap();

                for rank in ranks.iter() {
                    let (tax_id, sci_name) = nwr::find_rank(lineage, rank);
                    fields.push(sci_name.to_string());
                    if is_id {
                        fields.push(tax_id.to_string());
                    }
                }
            }

            let new_line: String = fields.join("\t");
            writer.write_fmt(format_args!("{}\n", new_line))?;
        }
    }
    writer.flush()?;

    Ok(())
}

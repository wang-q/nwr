use super::args;
use clap::{Arg, ArgAction, ArgMatches, Command};
use log::warn;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::io::Write;

/// Create clap subcommand arguments.
#[must_use]
pub fn make_subcommand() -> Command {
    Command::new("restrict")
        .about("Restricts taxonomy terms to ancestral descendants")
        .after_help(include_str!("../../docs/help/restrict.md"))
        .arg(args::terms_arg("The ancestor(s)"))
        .arg(args::dir_arg())
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .num_args(1..)
                .action(ArgAction::Append)
                .default_value("stdin")
                .help("Input filename. 'stdin' for standard input"),
        )
        .arg(args::column_arg())
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("exclude lines matching terms"),
        )
        .arg(args::outfile_arg())
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let nwrdir = nwr::get_nwr_dir(args, "dir")?;

    let column: usize = *args
        .get_one("column")
        .ok_or_else(|| anyhow::anyhow!("Missing 'column' argument"))?;
    let is_exclude = args.get_flag("exclude");

    let terms: Vec<String> = args
        .get_many::<String>("terms")
        .ok_or_else(|| anyhow::anyhow!("No terms provided"))?
        .cloned()
        .collect();

    let files: Vec<String> = args
        .get_many::<String>("file")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    let outfile = args
        .get_one::<String>("outfile")
        .ok_or_else(|| anyhow::anyhow!("Missing 'outfile' argument"))?;

    let mut writer = nwr::libs::io::writer(outfile)?;

    let conn = nwr::connect_txdb(&nwrdir)?;

    let mut id_set = HashSet::new();
    for term in &terms {
        let id = nwr::term_to_tax_id(&conn, term)?;
        let descendents = nwr::get_all_descendent(&conn, id)?;
        id_set.extend(descendents);
    }

    // Cache term lookups so that input files with duplicate terms don't
    // trigger redundant SQL queries. Failed lookups are also cached so that
    // repeated invalid terms skip without re-querying.
    let mut term_cache: HashMap<String, i64> = HashMap::new();
    let mut term_failed: HashSet<String> = HashSet::new();

    for infile in &files {
        let reader = nwr::libs::io::reader(infile)?;
        for (line_idx, line) in reader.lines().enumerate() {
            let line = line?;

            // Always output lines start with "#"
            if line.starts_with('#') {
                writer.write_fmt(format_args!("{line}\n"))?;
                continue;
            }

            // Check the given field
            let fields: Vec<&str> = line.split('\t').collect();
            let term = fields.get(column - 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "{}:{}: Column {} not found in line: {}",
                    infile,
                    line_idx + 1,
                    column,
                    line
                )
            })?;
            if term_failed.contains(*term) {
                continue;
            }
            let id = match term_cache.get(*term) {
                Some(&id) => id,
                None => match nwr::term_to_tax_id(&conn, term) {
                    Ok(x) => {
                        term_cache.insert((*term).to_string(), x);
                        x
                    }
                    Err(err) => {
                        warn!("Error converting term '{term}': {err}");
                        term_failed.insert((*term).to_string());
                        continue;
                    }
                },
            };

            if is_exclude ^ id_set.contains(&id) {
                writer.write_fmt(format_args!("{line}\n"))?;
            }
        }
    }
    writer.flush()?;

    Ok(())
}

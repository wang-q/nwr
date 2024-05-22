use clap::*;
use cmd_lib::*;
use itertools::Itertools;
use std::io::{BufRead, Write};
use std::{env, fs};
use tempfile::TempDir;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("pl-condense")
        .about("Pipeline - condense subtrees based on taxonomy")
        .after_help(
            r###"
* When <replace.tsv> is not provided, node names will be treated as taxonomic terms
* <replace.tsv> is a tab-separated file containing two fields

    node_name   taxonomy_id/scientific_name

* If `--rank` is empty, monophyletic subtree with the same taxonomic terms will be condensed
* When set one or more `--rank`, condense monophyletic nodes with the same ancestor

* This pipeline depends on the executable `nwr` itself

"###,
        )
        .arg(
            Arg::new("infile")
                .required(true)
                .num_args(1)
                .index(1)
                .help("Input filename. [stdin] for standard input"),
        )
        .arg(
            Arg::new("replace.tsv")
                .num_args(1)
                .index(2)
                .help("Path to replace.tsv"),
        )
        .arg(
            Arg::new("rank")
                .long("rank")
                .short('r')
                .num_args(1)
                .action(ArgAction::Append)
                .value_parser([
                    builder::PossibleValue::new("species"),
                    builder::PossibleValue::new("genus"),
                    builder::PossibleValue::new("family"),
                    builder::PossibleValue::new("order"),
                    builder::PossibleValue::new("class"),
                ])
                .help("According to which rank(s)"),
        )
        .arg(
            Arg::new("map")
                .long("map")
                .action(ArgAction::SetTrue)
                .help("Write a map file `condensed.tsv`"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Args
    //----------------------------
    let outfile = args.get_one::<String>("outfile").unwrap();

    let curdir = env::current_dir()?;
    let nwr = env::current_exe().unwrap().display().to_string();
    let tempdir = TempDir::new().unwrap();
    let tempdir_str = tempdir.path().to_str().unwrap();

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.to_string());
        }
    }

    run_cmd!(info "==> Paths")?;
    run_cmd!(info "    \"nwr\"     = ${nwr}")?;
    run_cmd!(info "    \"curdir\"  = ${curdir}")?;
    run_cmd!(info "    \"tempdir\" = ${tempdir_str}")?;

    //----------------------------
    // Operating
    //----------------------------
    run_cmd!(info "==> Absolute paths")?;
    let infile = args.get_one::<String>("infile").unwrap();
    let abs_infile = if infile == "stdin" {
        "stdin".to_string()
    } else {
        intspan::absolute_path(infile)
            .unwrap()
            .display()
            .to_string()
    };
    let mut abs_replace = if args.contains_id("replace.tsv") {
        intspan::absolute_path(args.get_one::<String>("replace.tsv").unwrap())
            .unwrap()
            .display()
            .to_string()
    } else {
        "dup".to_string()
    };

    run_cmd!(info "==> Switch to tempdir")?;
    env::set_current_dir(tempdir_str)?;

    run_cmd!(info "==> Start")?;
    run_cmd!(
        ${nwr} indent ${abs_infile} -o start.nwk
    )?;

    run_cmd!(info "==> Labels in the file")?;
    run_cmd!(
        ${nwr} label start.nwk -o labels.lst
    )?;

    if abs_replace == *"dup" {
        run_cmd!(info "==> Create replace.tsv from leaf labels")?;
        run_cmd!(
            ${nwr} label start.nwk -I -c dup -o replace.tsv
        )?;
        abs_replace = "replace.tsv".to_string();
    }

    run_cmd!(info "==> Add taxonomy info to the tree")?;
    run_cmd!(
        ${nwr} replace --mode species -I start.nwk ${abs_replace} -o commented.nwk
    )?;

    run_cmd!(info "==> Build groups")?;
    let mut groups = vec![];

    if ranks.is_empty() {
        for line in intspan::reader(&abs_replace).lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.split('\t').collect();
            if let Some(term) = parts.get(1) {
                groups.push(term.to_string());
            }
        }
    } else {
        let conn = nwr::connect_txdb(&nwr::nwr_path()).unwrap();

        for rank in ranks.iter() {
            'line: for line in
                intspan::reader(&abs_replace).lines().map_while(Result::ok)
            {
                let parts: Vec<&str> = line.split('\t').collect();
                if let Some(term) = parts.get(1) {
                    let id = match nwr::term_to_tax_id(&conn, term) {
                        Ok(x) => x,
                        Err(_) => continue 'line,
                    };
                    let lineage = match nwr::get_lineage(&conn, id) {
                        Err(_) => {
                            continue 'line;
                        }
                        Ok(x) => x,
                    };
                    let (_, sci_name) = nwr::find_rank(&lineage, rank.to_string());
                    groups.push(sci_name);
                }
            }
        }
    }

    groups = groups.into_iter().unique().filter(|s| s.ne("NA")).collect();

    run_cmd!(info "==> Condensing")?;
    let mut cur_tree = "commented.nwk".to_string();
    let mut condensed = vec![];
    for group in groups.iter() {
        let labels: Vec<String> = run_fun!(
            ${nwr} label ${cur_tree} -t ${group} --mode species -M
        )
        .unwrap()
        .split('\n')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

        if labels.is_empty() {
            continue;
        }

        let new_label = format!("{}___{}", group, labels.len());

        labels
            .iter()
            .for_each(|e| condensed.push(format!("{}\t{}", e, new_label)));

        run_cmd!(
            ${nwr} subtree ${cur_tree} -t ${group} --mode species -M -c ${new_label} -o condense.${group}.nwk
        )?;

        cur_tree = format!("condense.{}.nwk", group);
    }

    run_cmd!(info "==> Results")?;
    fs::copy(
        tempdir.path().join(cur_tree.clone()).to_str().unwrap(),
        "result.nwk",
    )?;

    let mut writer = intspan::writer("condensed.tsv");
    for line in condensed.iter() {
        writer.write_all(format!("{}\n", line).as_ref())?;
    }
    writer.flush()?;

    //----------------------------
    // Done
    //----------------------------
    if outfile == "stdout" {
        run_cmd!(cat ${cur_tree})?;
        env::set_current_dir(&curdir)?;
    } else {
        env::set_current_dir(&curdir)?;
        fs::copy(tempdir.path().join("result.nwk").to_str().unwrap(), outfile)?;
    }

    if args.get_flag("map") {
        fs::copy(
            tempdir.path().join("condensed.tsv").to_str().unwrap(),
            "condensed.tsv",
        )?;
    }

    Ok(())
}

// use std::io::Read
// fn pause() {
//     let mut stdin = std::io::stdin();
//     let mut stdout = std::io::stdout();
//
//     // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
//     write!(stdout, "Press any key to continue...").unwrap();
//     stdout.flush().unwrap();
//
//     // Read a single byte and discard
//     let _ = stdin.read(&mut [0u8]).unwrap();
// }

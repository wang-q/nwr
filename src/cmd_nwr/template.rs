use super::args;
use clap::*;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::BufRead;
use tera::{Context, Tera};

/// Create clap subcommand arguments.
pub fn make_subcommand() -> Command {
    Command::new("template")
        .about("Creates dirs, data and scripts for a phylogenomic research")
        .after_help(include_str!("../../docs/help/template.md"))
        // Global
        .arg(args::infiles_arg(".assembly.tsv files"))
        .arg(args::outdir_arg())
        .arg(
            Arg::new("include")
                .long("include")
                .num_args(1..)
                .action(ArgAction::Append)
                .help(
                    "Only the assemblies *in* these lists in the MinHash, Count and Protein steps",
                ),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Only the assemblies *not in* these lists"),
        )
        .arg(
            Arg::new("parallel")
                .long("parallel")
                .num_args(1)
                .default_value("8")
                .value_parser(value_parser!(usize))
                .help("Number of threads"),
        )
        // ASSEMBLY
        .arg(
            Arg::new("ass")
                .long("ass")
                .action(ArgAction::SetTrue)
                .help("Prepare ASSEMBLY/ materials"),
        )
        // BioSample
        .arg(
            Arg::new("bs")
                .long("bs")
                .action(ArgAction::SetTrue)
                .help("Prepare BioSample/ materials"),
        )
        // MinHash
        .arg(
            Arg::new("mh")
                .long("mh")
                .action(ArgAction::SetTrue)
                .help("Prepare MinHash/ materials"),
        )
        .arg(
            Arg::new("sketch")
                .long("sketch")
                .num_args(1)
                .default_value("10000")
                .value_parser(value_parser!(usize))
                .help("Sketch size passed to `mash sketch`"),
        )
        .arg(
            Arg::new("ani-ab")
                .long("ani-ab")
                .num_args(1)
                .default_value("0.05")
                .value_parser(value_parser!(f64))
                .help("The ANI value for abnormal strains"),
        )
        .arg(
            Arg::new("ani-nr")
                .long("ani-nr")
                .num_args(1)
                .default_value("0.005")
                .value_parser(value_parser!(f64))
                .help("The ANI value for non-redundant strains"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .num_args(1)
                .default_value("0.5")
                .value_parser(value_parser!(f64))
                .help("Height value passed to `cutree()`"),
        )
        // Count
        .arg(
            Arg::new("count")
                .long("count")
                .action(ArgAction::SetTrue)
                .help("Prepare Count/ materials"),
        )
        .arg(args::rank_arg())
        .arg(
            Arg::new("lineage")
                .long("lineage")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("To list which rank(s) in the lineage"),
        )
        // Protein
        .arg(
            Arg::new("pro")
                .long("pro")
                .action(ArgAction::SetTrue)
                .help("Prepare Protein/ materials"),
        )
}

/// Command implementation.
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let outdir = args.get_one::<String>("outdir").unwrap();
    let stdout_mode = outdir == nwr::libs::template::STDOUT_MARKER;

    let ins: Vec<String> = args
        .get_many::<String>("include")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let not_ins: Vec<String> = args
        .get_many::<String>("exclude")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let infiles: Vec<String> = args
        .get_many::<String>("infiles")
        .ok_or_else(|| anyhow::anyhow!("No input files provided"))?
        .cloned()
        .collect();

    let ranks: Vec<String> = args
        .get_many::<String>("rank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let lineages: Vec<String> = args
        .get_many::<String>("lineage")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let parallel = *args.get_one::<usize>("parallel").unwrap();
    let sketch = *args.get_one::<usize>("sketch").unwrap();
    let ani_ab = *args.get_one::<f64>("ani-ab").unwrap();
    let ani_nr = *args.get_one::<f64>("ani-nr").unwrap();
    let height = *args.get_one::<f64>("height").unwrap();
    let do_ass = args.get_flag("ass");
    let do_bs = args.get_flag("bs");
    let do_mh = args.get_flag("mh");
    let do_count = args.get_flag("count");
    let do_pro = args.get_flag("pro");

    if stdout_mode {
        eprintln!(
            "Warning: stdout mode produces concatenated output from multiple files"
        );
    } else {
        fs::create_dir_all(outdir)?;
    }

    let mut ass_url_of = BTreeMap::new();
    let mut ass_species_of = BTreeMap::new();

    let mut bs_name_of = BTreeMap::new();
    let mut bs_species_of = BTreeMap::new();

    let mut mh_species_of = BTreeMap::new();
    let mut mh_level_of = BTreeMap::new();

    let mut count_species_of = BTreeMap::new();

    let mut pro_species_of = BTreeMap::new();

    // Track duplicate strain names across all input files so that the warning
    // is emitted once regardless of which name-keyed stages (ass, mh, count,
    // pro) are enabled.
    let mut seen_names: HashSet<String> = HashSet::new();

    for infile in &infiles {
        let reader = nwr::libs::io::reader(infile)?;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();

            if fields.len() < 5 {
                return Err(anyhow::anyhow!(
                    "{}:{}: Line has {} fields, expected at least 5: {}",
                    infile,
                    line_num + 1,
                    fields.len(),
                    line
                ));
            }

            let name = nwr::libs::template::validate_shell_safe(fields[0])
                .map_err(|e| anyhow::anyhow!("{}:{}: {}", infile, line_num + 1, e))?;
            let url = nwr::libs::template::validate_no_control_chars(fields[1])
                .map_err(|e| anyhow::anyhow!("{}:{}: {}", infile, line_num + 1, e))?;
            let sample = fields[2];
            let sample = if sample.is_empty() {
                sample
            } else {
                nwr::libs::template::validate_shell_safe(sample)
                    .map_err(|e| anyhow::anyhow!("{}:{}: {}", infile, line_num + 1, e))?
            };

            // format species strings
            let species = nwr::libs::template::validate_no_control_chars(fields[3])
                .map_err(|e| anyhow::anyhow!("{}:{}: {}", infile, line_num + 1, e))?;
            let species_formatted = nwr::libs::template::format_species_name(species);
            let species_ = nwr::libs::template::validate_shell_safe(&species_formatted)
                .map_err(|e| anyhow::anyhow!("{}:{}: {}", infile, line_num + 1, e))?;

            let level = match fields[4] {
                "Complete Genome" => nwr::libs::template::LEVEL_COMPLETE_GENOME,
                "Chromosome" => nwr::libs::template::LEVEL_CHROMOSOME,
                "Scaffold" => nwr::libs::template::LEVEL_SCAFFOLD,
                "Contig" => nwr::libs::template::LEVEL_CONTIG,
                _ => nwr::libs::template::LEVEL_OTHER,
            };

            // Warn once about duplicate strain names across all name-keyed
            // stages (ass, mh, count, pro) so users learn about duplicates
            // even when --ass is not set.
            if (do_ass || do_mh || do_count || do_pro)
                && !seen_names.insert(name.to_string())
            {
                eprintln!(
                    "Warning: duplicate strain name '{}', overwriting previous entry",
                    name
                );
            }

            // ass
            if do_ass {
                ass_url_of.insert(name.to_string(), url.to_string());
                ass_species_of.insert(name.to_string(), species_.to_string());
            }

            // bs (keyed by `sample`, so it has its own duplicate check)
            if do_bs && !sample.is_empty() {
                if bs_name_of.contains_key(sample) {
                    eprintln!(
                        "Warning: duplicate sample name '{}', overwriting previous entry",
                        sample
                    );
                }
                bs_name_of.insert(sample.to_string(), name.to_string());
                bs_species_of.insert(sample.to_string(), species_.to_string());
            }

            // mh
            if do_mh {
                mh_species_of.insert(name.to_string(), species_.to_string());
                mh_level_of.insert(name.to_string(), level.to_string());
            }

            // count
            if do_count {
                count_species_of.insert(name.to_string(), species_.to_string());
            }

            // pro
            if do_pro {
                pro_species_of.insert(name.to_string(), species_.to_string());
            }
        }
    }

    // column index in strains.taxon.tsv
    let mut rank_col_of = BTreeMap::new();
    rank_col_of.insert("genus".to_string(), "3".to_string());
    rank_col_of.insert("family".to_string(), "4".to_string());
    rank_col_of.insert("order".to_string(), "5".to_string());
    rank_col_of.insert("class".to_string(), "6".to_string());

    //----------------------------
    // Context
    //----------------------------
    let mut context = Context::new();

    context.insert("outdir", outdir);
    context.insert("ins", &ins);
    context.insert("not_ins", &not_ins);
    context.insert("parallel", &parallel.to_string());

    context.insert("ass_url_of", &ass_url_of);
    context.insert("ass_species_of", &ass_species_of);

    context.insert("bs_name_of", &bs_name_of);
    context.insert("bs_species_of", &bs_species_of);

    context.insert("mh_species_of", &mh_species_of);
    context.insert("mh_level_of", &mh_level_of);
    context.insert("mh_sketch", &sketch.to_string());
    context.insert("mh_ani_ab", &ani_ab.to_string());
    context.insert("mh_ani_nr", &ani_nr.to_string());
    context.insert("mh_height", &height.to_string());

    context.insert("count_species_of", &count_species_of);
    context.insert("count_ranks", &ranks);
    context.insert("count_lineages", &lineages);
    context.insert("rank_col_of", &rank_col_of);

    context.insert("pro_species_of", &pro_species_of);

    let ass_columns = vec![
        "Organism_name",
        "Taxid",
        "Assembly_name",
        "Infraspecific_name",
        "BioSample",
        "BioProject",
        "Submitter",
        "Date",
        "Assembly_type",
        "Release_type",
        "Assembly_level",
        "Genome_representation",
        "WGS_project",
        "Assembly_method",
        "Genome_coverage",
        "Sequencing_technology",
        "RefSeq_category",
        "RefSeq_assembly_accession",
        "GenBank_assembly_accession",
    ];
    context.insert("ass_columns", &ass_columns);

    //----------------------------
    // Writing
    //----------------------------
    let mut tera = Tera::default();
    tera.add_raw_template("header", include_str!("../../templates/header.tera.sh"))?;

    if do_ass {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/ASSEMBLY", outdir))?;
        }
        nwr::libs::template::gen_ass_data(&context)?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/ass_aria2.tera.sh"),
            "ASSEMBLY",
            "aria2.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/ass_check.tera.sh"),
            "ASSEMBLY",
            "check.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/ass_reorder.tera.sh"),
            "ASSEMBLY",
            "reorder.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/ass_n50.tera.sh"),
            "ASSEMBLY",
            "n50.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/ass_collect.tera.sh"),
            "ASSEMBLY",
            "collect.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/ass_finish.tera.sh"),
            "ASSEMBLY",
            "finish.sh",
        )?;
    }

    if do_bs {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/BioSample", outdir))?;
        }
        nwr::libs::template::gen_bs_data(&context)?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/bs_download.tera.sh"),
            "BioSample",
            "download.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/bs_collect.tera.sh"),
            "BioSample",
            "collect.sh",
        )?;
    }

    if do_mh {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/MinHash", outdir))?;
        }
        nwr::libs::template::gen_mh_data(&context)?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/mh_compute.tera.sh"),
            "MinHash",
            "compute.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/mh_nr.tera.sh"),
            "MinHash",
            "nr.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/mh_abnormal.tera.sh"),
            "MinHash",
            "abnormal.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/mh_dist.tera.sh"),
            "MinHash",
            "dist.sh",
        )?;
    }

    if do_count {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/Count", outdir))?;
        }
        nwr::libs::template::gen_count_data(&context)?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/count_strains.tera.sh"),
            "Count",
            "strains.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/count_rank.tera.sh"),
            "Count",
            "rank.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/count_lineage.tera.sh"),
            "Count",
            "lineage.sh",
        )?;
    }

    if do_pro {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/Protein", outdir))?;
        }
        nwr::libs::template::gen_pro_data(&context)?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/pro_collect.tera.sh"),
            "Protein",
            "collect.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/pro_cluster.tera.sh"),
            "Protein",
            "cluster.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/pro_info.tera.sh"),
            "Protein",
            "info.sh",
        )?;
        nwr::libs::template::render_shell_script(
            &mut tera,
            &context,
            include_str!("../../templates/pro_count.tera.sh"),
            "Protein",
            "count.sh",
        )?;
    }

    Ok(())
}

use super::args;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{BufRead, Write};
use tera::{Context, Tera};

/// Write a two-column `species.tsv` (`key<TAB>species`) from a single map in
/// the context. Used by the Count and Protein output stages.
fn write_species_tsv(
    context: &Context,
    subdir: &str,
    map_key: &str,
) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create {subdir}/{outname}");

    let outdir = nwr::libs::template::get_outdir(context)?;
    let map = context
        .get(map_key)
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing '{map_key}' in template context"))?;

    let mut writer = nwr::libs::template::open_writer(outdir, subdir, outname)?;
    for (key, value) in map {
        let species = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("'{map_key}' value for '{key}' is not a string")
        })?;
        writeln!(writer, "{key}\t{species}")?;
    }
    writer.flush()?;
    writer.finish()?;
    Ok(())
}

/// Generate ASSEMBLY/url.tsv.
fn gen_ass_data(context: &Context) -> anyhow::Result<()> {
    let outname = "url.tsv";
    eprintln!("Create ASSEMBLY/{outname}");

    let outdir = nwr::libs::template::get_outdir(context)?;
    let ass_url_of = context
        .get("ass_url_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'ass_url_of' in template context"))?;
    let ass_species_of = context
        .get("ass_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            anyhow::anyhow!("Missing 'ass_species_of' in template context")
        })?;

    let mut writer = nwr::libs::template::open_writer(outdir, "ASSEMBLY", outname)?;
    for (key, value) in ass_url_of {
        let url = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("ass_url_of value for '{key}' is not a string")
        })?;
        let species = ass_species_of
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "ass_species_of value for '{key}' is missing or not a string"
                )
            })?;

        writeln!(writer, "{key}\t{url}\t{species}")?;
    }
    writer.flush()?;
    writer.finish()?;

    Ok(())
}

/// Generate BioSample/sample.tsv.
fn gen_bs_data(context: &Context) -> anyhow::Result<()> {
    let outname = "sample.tsv";
    eprintln!("Create BioSample/{outname}");

    let outdir = nwr::libs::template::get_outdir(context)?;
    let bs_name_of = context
        .get("bs_name_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'bs_name_of' in template context"))?;
    let bs_species_of = context
        .get("bs_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'bs_species_of' in template context"))?;

    let mut writer = nwr::libs::template::open_writer(outdir, "BioSample", outname)?;

    for (key, value) in bs_name_of {
        let name = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("bs_name_of value for '{key}' is not a string")
        })?;
        let species =
            bs_species_of
                .get(key)
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bs_species_of value for '{key}' is missing or not a string"
                    )
                })?;

        writeln!(writer, "{key}\t{name}\t{species}")?;
    }
    writer.flush()?;
    writer.finish()?;

    Ok(())
}

/// Generate MinHash/species.tsv.
fn gen_mh_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create MinHash/{outname}");

    let outdir = nwr::libs::template::get_outdir(context)?;
    let mh_species_of = context
        .get("mh_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'mh_species_of' in template context"))?;
    let mh_level_of = context
        .get("mh_level_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'mh_level_of' in template context"))?;

    let mut writer = nwr::libs::template::open_writer(outdir, "MinHash", outname)?;

    for (key, value) in mh_species_of {
        let species = value.as_str().ok_or_else(|| {
            anyhow::anyhow!("mh_species_of value for '{key}' is not a string")
        })?;
        let level = mh_level_of
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "mh_level_of value for '{key}' is missing or not a string"
                )
            })?;

        writeln!(writer, "{key}\t{species}\t{level}")?;
    }
    writer.flush()?;
    writer.finish()?;

    Ok(())
}

/// Generate Count/species.tsv.
fn gen_count_data(context: &Context) -> anyhow::Result<()> {
    write_species_tsv(context, "Count", "count_species_of")
}

/// Generate Protein/species.tsv.
fn gen_pro_data(context: &Context) -> anyhow::Result<()> {
    write_species_tsv(context, "Protein", "pro_species_of")
}

/// Ranks supported by the Count stage.
///
/// These are the only ranks that have a fixed column index in
/// `strains.taxon.tsv`; passing anything else to `--rank` or `--lineage`
/// would generate invalid shell scripts.
const VALID_COUNT_RANKS: &[&str] = &["genus", "family", "order", "class"];

/// Create clap subcommand arguments.
#[must_use]
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
                .value_parser(clap::builder::RangedU64ValueParser::<usize>::new().range(1..))
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
                .value_parser(clap::builder::RangedU64ValueParser::<usize>::new().range(1..))
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
    const ASS_COLUMNS: &[&str] = &[
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

    let outdir = args
        .get_one::<String>("outdir")
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' argument"))?;
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

    // Ranks and lineages are interpolated into generated shell scripts and Perl
    // variable names; restrict them to the supported set to avoid broken
    // scripts or accidental shell injection.
    for rank in &ranks {
        if !VALID_COUNT_RANKS.contains(&rank.as_str()) {
            anyhow::bail!(
                "Invalid rank '{rank}' for --count. Valid ranks are: {VALID_COUNT_RANKS:?}"
            );
        }
    }
    for rank in &lineages {
        if !VALID_COUNT_RANKS.contains(&rank.as_str()) {
            anyhow::bail!(
                "Invalid rank '{rank}' for --lineage. Valid ranks are: {VALID_COUNT_RANKS:?}"
            );
        }
    }

    // Include/exclude paths are embedded unquoted in generated scripts; reject
    // values that would break the script or be interpreted as shell syntax.
    for path in &ins {
        nwr::libs::template::validate_path_safe(path)
            .map_err(|e| anyhow::anyhow!("Invalid --include path: {e}"))?;
    }
    for path in &not_ins {
        nwr::libs::template::validate_path_safe(path)
            .map_err(|e| anyhow::anyhow!("Invalid --exclude path: {e}"))?;
    }

    let parallel = *args
        .get_one::<usize>("parallel")
        .ok_or_else(|| anyhow::anyhow!("Missing 'parallel' argument"))?;
    let sketch = *args
        .get_one::<usize>("sketch")
        .ok_or_else(|| anyhow::anyhow!("Missing 'sketch' argument"))?;
    let ani_ab = *args
        .get_one::<f64>("ani-ab")
        .ok_or_else(|| anyhow::anyhow!("Missing 'ani-ab' argument"))?;
    let ani_nr = *args
        .get_one::<f64>("ani-nr")
        .ok_or_else(|| anyhow::anyhow!("Missing 'ani-nr' argument"))?;
    let height = *args
        .get_one::<f64>("height")
        .ok_or_else(|| anyhow::anyhow!("Missing 'height' argument"))?;
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
            if line.trim().is_empty() {
                continue;
            }
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
            let species_formatted = nwr::libs::abbr::clean_name(species);
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
                    "Warning: duplicate strain name '{name}', overwriting previous entry"
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
                        "Warning: duplicate sample name '{sample}', overwriting previous entry"
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
    for (i, rank) in VALID_COUNT_RANKS.iter().enumerate() {
        // strains.taxon.tsv columns: 1=name, 2=species, 3=genus, 4=family, ...
        rank_col_of.insert((*rank).to_string(), (i + 3).to_string());
    }

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

    context.insert("ass_columns", &ASS_COLUMNS);

    //----------------------------
    // Writing
    //----------------------------
    let mut tera = Tera::default();
    tera.add_raw_template("header", include_str!("../../templates/header.tera.sh"))?;

    if do_ass {
        if !stdout_mode {
            fs::create_dir_all(format!("{outdir}/ASSEMBLY"))?;
        }
        gen_ass_data(&context)?;
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
            fs::create_dir_all(format!("{outdir}/BioSample"))?;
        }
        gen_bs_data(&context)?;
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
            fs::create_dir_all(format!("{outdir}/MinHash"))?;
        }
        gen_mh_data(&context)?;
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
            fs::create_dir_all(format!("{outdir}/Count"))?;
        }
        gen_count_data(&context)?;
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
            fs::create_dir_all(format!("{outdir}/Protein"))?;
        }
        gen_pro_data(&context)?;
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

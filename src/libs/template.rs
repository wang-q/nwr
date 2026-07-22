use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use std::io::Write;
use tera::{Context, Tera};

/// Assembly level code for a complete genome.
pub const LEVEL_COMPLETE_GENOME: &str = "1";
/// Assembly level code for a chromosome-level assembly.
pub const LEVEL_CHROMOSOME: &str = "2";
/// Assembly level code for a scaffold-level assembly.
pub const LEVEL_SCAFFOLD: &str = "3";
/// Assembly level code for a contig-level assembly (same as scaffold here).
pub const LEVEL_CONTIG: &str = "3"; // Same as SCAFFOLD - both are treated as level 3
/// Assembly level code for other incomplete assemblies.
pub const LEVEL_OTHER: &str = "5";

lazy_static! {
    static ref RE_S1: Regex = Regex::new(r#"(?xi)\W+"#).unwrap();
    static ref RE_S2: Regex = Regex::new(r#"(?xi)_+"#).unwrap();
    static ref RE_S3: Regex = Regex::new(r#"(?xi)_$"#).unwrap();
    static ref RE_S4: Regex = Regex::new(r#"(?xi)^_"#).unwrap();
    static ref RE_URL: Regex =
        Regex::new(r#"(?xi)(ftp|https?)://ftp.ncbi.nlm.nih.gov/"#).unwrap();
}

/// Validate that a string is safe to embed into generated shell scripts and
/// to use as a file or directory name. Only ASCII alphanumeric characters,
/// underscores, hyphens and dots are allowed.
pub fn validate_shell_safe(s: &str) -> anyhow::Result<&str> {
    if s.is_empty() {
        return Err(anyhow::anyhow!("Shell-safe identifier must not be empty"));
    }
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(s)
    } else {
        Err(anyhow::anyhow!(
            "Identifier contains characters unsafe for shell usage: '{}'",
            s
        ))
    }
}

/// Reject strings that would corrupt TSV output or be unsafe in shell contexts.
/// This project only supports NCBI URLs; non-NCBI URLs are intentionally out of scope.
pub fn validate_no_control_chars(s: &str) -> anyhow::Result<&str> {
    if s.chars().any(|c| c.is_ascii_control()) {
        return Err(anyhow::anyhow!(
            "String contains control characters: '{}'",
            s
        ));
    }
    Ok(s)
}

/// Marker value for stdout output mode.
pub const STDOUT_MARKER: &str = "stdout";

/// Create a writer for the given output location.
/// When `outdir` equals `STDOUT_MARKER`, writes to stdout; otherwise writes
/// to `{outdir}/{subdir}/{outname}`.
pub fn open_writer(
    outdir: &str,
    subdir: &str,
    outname: &str,
) -> Box<dyn std::io::Write> {
    if outdir == STDOUT_MARKER {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/{}/{}", outdir, subdir, outname).as_ref())
    }
}

/// Parsed options for template generation.
pub struct TemplateOptions {
    /// Output directory (or "stdout" for dry-run output).
    pub outdir: String,
    /// Input TSV files containing strain information.
    pub infiles: Vec<String>,
    /// Strain names to include in the pipeline.
    pub ins: Vec<String>,
    /// Strain names to exclude from the pipeline.
    pub not_ins: Vec<String>,
    /// Number of parallel workers.
    pub parallel: usize,
    /// Mash sketch size.
    pub sketch: usize,
    /// ANI threshold for alignment blocks.
    pub ani_ab: f64,
    /// ANI threshold for non-redundant clustering.
    pub ani_nr: f64,
    /// Maximum tree height for visualization.
    pub height: f64,
    /// Taxonomic ranks to include in reports.
    pub ranks: Vec<String>,
    /// Lineage ranks to output in annotations.
    pub lineages: Vec<String>,
    /// Generate assembly scripts.
    pub do_ass: bool,
    /// Generate BioSample materials.
    pub do_bs: bool,
    /// Generate mash sketches.
    pub do_mh: bool,
    /// Generate count matrices.
    pub do_count: bool,
    /// Generate protein-related scripts.
    pub do_pro: bool,
}

/// Generate directories, data files and shell scripts for phylogenomic research.
pub fn run(options: &TemplateOptions) -> anyhow::Result<()> {
    let outdir = &options.outdir;
    let stdout_mode = outdir == STDOUT_MARKER;
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

    for infile in &options.infiles {
        let reader = intspan::reader(infile);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();

            if fields.len() < 5 {
                return Err(anyhow::anyhow!(
                    "Line has {} fields, expected at least 5: {}",
                    fields.len(),
                    line
                ));
            }

            let name = validate_shell_safe(fields[0])?;
            let url = validate_no_control_chars(fields[1])?;
            let sample = fields[2];
            let sample = if sample.is_empty() {
                sample
            } else {
                validate_shell_safe(sample)?
            };

            // format species strings
            let species = validate_no_control_chars(fields[3])?;
            let s1 = RE_S1.replace_all(species, "_");
            let s2 = RE_S2.replace_all(&s1, "_");
            let s3 = RE_S3.replace_all(&s2, "");
            let s4 = RE_S4.replace_all(&s3, "");
            let species_formatted = s4.to_string();
            let species_ = validate_shell_safe(&species_formatted)?;

            let level = match fields[4] {
                "Complete Genome" => LEVEL_COMPLETE_GENOME,
                "Chromosome" => LEVEL_CHROMOSOME,
                "Scaffold" => LEVEL_SCAFFOLD,
                "Contig" => LEVEL_CONTIG,
                _ => LEVEL_OTHER,
            };

            // ass
            if ass_url_of.contains_key(name) {
                eprintln!(
                    "Warning: duplicate strain name '{}', overwriting previous entry",
                    name
                );
            }
            ass_url_of.insert(name.to_string(), url.to_string());
            ass_species_of.insert(name.to_string(), species_.to_string());

            // bs
            if !sample.is_empty() {
                bs_name_of.insert(sample.to_string(), name.to_string());
                bs_species_of.insert(sample.to_string(), species_.to_string());
            }

            // mh
            mh_species_of.insert(name.to_string(), species_.to_string());
            mh_level_of.insert(name.to_string(), level.to_string());

            // count
            count_species_of.insert(name.to_string(), species.to_string());

            // pro
            pro_species_of.insert(name.to_string(), species_.to_string());
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
    context.insert("ins", &options.ins);
    context.insert("not_ins", &options.not_ins);
    context.insert("parallel", &options.parallel.to_string());

    context.insert("ass_url_of", &ass_url_of);
    context.insert("ass_species_of", &ass_species_of);

    context.insert("bs_name_of", &bs_name_of);
    context.insert("bs_species_of", &bs_species_of);

    context.insert("mh_species_of", &mh_species_of);
    context.insert("mh_level_of", &mh_level_of);
    context.insert("mh_sketch", &options.sketch.to_string());
    context.insert("mh_ani_ab", &options.ani_ab.to_string());
    context.insert("mh_ani_nr", &options.ani_nr.to_string());
    context.insert("mh_height", &options.height.to_string());

    context.insert("count_species_of", &count_species_of);
    context.insert("count_ranks", &options.ranks);
    context.insert("count_lineages", &options.lineages);
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
    if options.do_ass {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/ASSEMBLY", outdir))?;
        }
        gen_ass_data(&context)?;
        render_shell_script(
            &context,
            include_str!("../../templates/ass_aria2.tera.sh"),
            "ASSEMBLY",
            "aria2.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/ass_check.tera.sh"),
            "ASSEMBLY",
            "check.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/ass_reorder.tera.sh"),
            "ASSEMBLY",
            "reorder.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/ass_n50.tera.sh"),
            "ASSEMBLY",
            "n50.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/ass_collect.tera.sh"),
            "ASSEMBLY",
            "collect.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/ass_finish.tera.sh"),
            "ASSEMBLY",
            "finish.sh",
        )?;
    }

    if options.do_bs {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/BioSample", outdir))?;
        }
        gen_bs_data(&context)?;
        render_shell_script(
            &context,
            include_str!("../../templates/bs_download.tera.sh"),
            "BioSample",
            "download.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/bs_collect.tera.sh"),
            "BioSample",
            "collect.sh",
        )?;
    }

    if options.do_mh {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/MinHash", outdir))?;
        }
        gen_mh_data(&context)?;
        render_shell_script(
            &context,
            include_str!("../../templates/mh_compute.tera.sh"),
            "MinHash",
            "compute.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/mh_nr.tera.sh"),
            "MinHash",
            "nr.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/mh_abnormal.tera.sh"),
            "MinHash",
            "abnormal.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/mh_dist.tera.sh"),
            "MinHash",
            "dist.sh",
        )?;
    }

    if options.do_count {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/Count", outdir))?;
        }
        gen_count_data(&context)?;
        render_shell_script(
            &context,
            include_str!("../../templates/count_strains.tera.sh"),
            "Count",
            "strains.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/count_rank.tera.sh"),
            "Count",
            "rank.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/count_lineage.tera.sh"),
            "Count",
            "lineage.sh",
        )?;
    }

    if options.do_pro {
        if !stdout_mode {
            fs::create_dir_all(format!("{}/Protein", outdir))?;
        }
        gen_pro_data(&context)?;
        render_shell_script(
            &context,
            include_str!("../../templates/pro_collect.tera.sh"),
            "Protein",
            "collect.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/pro_cluster.tera.sh"),
            "Protein",
            "cluster.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/pro_info.tera.sh"),
            "Protein",
            "info.sh",
        )?;
        render_shell_script(
            &context,
            include_str!("../../templates/pro_count.tera.sh"),
            "Protein",
            "count.sh",
        )?;
    }

    Ok(())
}

/// Helper function to render shell scripts from Tera templates.
pub fn render_shell_script(
    context: &Context,
    template_content: &str,
    subdir: &str,
    outname: &str,
) -> anyhow::Result<()> {
    eprintln!("Create {}/{}", subdir, outname);

    let outdir = context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))?;

    let mut writer = open_writer(outdir, subdir, outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", template_content),
    ])?;

    let rendered = tera.render("t", context)?;
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

/// Generate ASSEMBLY/url.tsv and url_rsync.tsv.
pub fn gen_ass_data(context: &Context) -> anyhow::Result<()> {
    let outname = "url.tsv";
    let outname_rsync = "url_rsync.tsv";
    eprintln!("Create ASSEMBLY/{}", outname);
    eprintln!("Create ASSEMBLY/{}", outname_rsync);

    let outdir = context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))?;
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

    let mut writer = open_writer(outdir, "ASSEMBLY", outname);

    for (key, value) in ass_url_of {
        let url = value.as_str().unwrap();
        let species = ass_species_of.get(key).unwrap().as_str().unwrap();

        writer.write_all(format!("{}\t{}\t{}\n", key, url, species).as_ref())?;
    }

    let mut writer_rsync = open_writer(outdir, "ASSEMBLY", outname_rsync);
    for (key, value) in ass_url_of {
        let url = value.as_str().unwrap();
        let species = ass_species_of.get(key).unwrap().as_str().unwrap();

        let rsync = RE_URL.replace(url, "ftp.ncbi.nlm.nih.gov::");
        writer_rsync.write_all(format!("{}\t{}\t{}\n", key, rsync, species).as_ref())?;
    }

    Ok(())
}

/// Generate BioSample/sample.tsv.
pub fn gen_bs_data(context: &Context) -> anyhow::Result<()> {
    let outname = "sample.tsv";
    eprintln!("Create BioSample/{}", outname);

    let outdir = context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))?;
    let bs_name_of = context
        .get("bs_name_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'bs_name_of' in template context"))?;
    let bs_species_of = context
        .get("bs_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'bs_species_of' in template context"))?;

    let mut writer = open_writer(outdir, "BioSample", outname);

    for (key, value) in bs_name_of {
        let name = value.as_str().unwrap();
        let species = bs_species_of.get(key).unwrap().as_str().unwrap();

        writer.write_all(format!("{}\t{}\t{}\n", key, name, species).as_ref())?;
    }

    Ok(())
}

/// Generate MinHash/species.tsv.
pub fn gen_mh_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))?;
    let mh_species_of = context
        .get("mh_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'mh_species_of' in template context"))?;
    let mh_level_of = context
        .get("mh_level_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("Missing 'mh_level_of' in template context"))?;

    let mut writer = open_writer(outdir, "MinHash", outname);

    for (key, value) in mh_species_of {
        let species = value.as_str().unwrap();
        let level = mh_level_of.get(key).unwrap().as_str().unwrap();

        writer.write_all(format!("{}\t{}\t{}\n", key, species, level).as_ref())?;
    }

    Ok(())
}

/// Generate Count/species.tsv.
pub fn gen_count_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create Count/{}", outname);

    let outdir = context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))?;
    let count_species_of = context
        .get("count_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            anyhow::anyhow!("Missing 'count_species_of' in template context")
        })?;

    let mut writer = open_writer(outdir, "Count", outname);

    for (key, value) in count_species_of {
        let species = value.as_str().unwrap();

        writer.write_all(format!("{}\t{}\n", key, species).as_ref())?;
    }

    Ok(())
}

/// Generate Protein/species.tsv.
pub fn gen_pro_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create Protein/{}", outname);

    let outdir = context
        .get("outdir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'outdir' in template context"))?;
    let species_of = context
        .get("pro_species_of")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            anyhow::anyhow!("Missing 'pro_species_of' in template context")
        })?;

    let mut writer = open_writer(outdir, "Protein", outname);

    for (key, value) in species_of {
        let species = value.as_str().unwrap();

        writer.write_all(format!("{}\t{}\n", key, species).as_ref())?;
    }

    Ok(())
}

use clap::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use tera::{Context, Tera};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("template")
        .about("Create dirs, data and scripts for a phylogenomic research")
        .after_help(
            r###"
.assembly.tsv: a TAB delimited file to guide downloading and processing of files.

| Col |  Type  | Description                                              |
|----:|:------:|:---------------------------------------------------------|
|   1 | string | #name: species + infraspecific_name + assembly_accession |
|   2 | string | ftp_path                                                 |
|   3 | string | biosample                                                |
|   4 | string | species                                                  |
|   5 | string | assembly_level                                           |

* --ass: ASSEMBLY/
    * One TSV file
        * url.tsv
    * And five Bash scripts
        * rsync.sh
        * check.sh
        * n50.sh [LEN_N50] [N_CONTIG] [LEN_SUM]
        * collect.sh
        * finish.sh

* --bs: BioSample/
    * One TSV file
        * sample.tsv
    * And two Bash scripts
        * download.sh
        * collect.sh [N_ATTR]

* --mh: MinHash/
    * One TSV file
        * species.tsv
    * And five Bash scripts
        * compute.sh
        * species.sh
        * abnormal.sh
        * nr.sh
        * dist.sh

* --count: Count/
    * One TSV file
        * species.tsv
    * Three Bash scripts
        * strains.sh - strains.taxon.tsv, species, genus, family, order, and class
        * rank.sh - count species and strains
        * lineage.sh - count strains

* --pro: Protein/
    * One TSV file
        * species.tsv
    * Bash scripts
        * collect.sh
        * info.sh
        * count.sh

"###,
        )
        // Global
        .arg(
            Arg::new("infiles")
                .help(".assembly.tsv files")
                .num_args(1..)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("outdir")
                .long("outdir")
                .short('o')
                .num_args(1)
                .default_value(".")
                .help("Output directory. [stdout] for screen"),
        )
        .arg(
            Arg::new("in")
                .long("in")
                .num_args(1..)
                .action(ArgAction::Append)
                .help(
                    "Only the assemblies *in* these lists in the MinHash, Count and Protein steps",
                ),
        )
        .arg(
            Arg::new("not-in")
                .long("not-in")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Only the assemblies *not in* these lists"),
        )
        .arg(
            Arg::new("parallel")
                .long("parallel")
                .num_args(1)
                .default_value("8")
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
                .default_value("100000")
                .help("Sketch size passed to `mash sketch`"),
        )
        .arg(
            Arg::new("ani-ab")
                .long("ani-ab")
                .num_args(1)
                .default_value("0.05")
                .help("The ANI value for abnormal strains"),
        )
        .arg(
            Arg::new("ani-nr")
                .long("ani-nr")
                .num_args(1)
                .default_value("0.005")
                .help("The ANI value for non-redundant strains"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .num_args(1)
                .default_value("0.5")
                .help("Height value passed to `cutree()`"),
        )
        // Count
        .arg(
            Arg::new("count")
                .long("count")
                .action(ArgAction::SetTrue)
                .help("Prepare Count/ materials"),
        )
        .arg(
            Arg::new("rank")
                .long("rank")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("To list which rank(s) - genus, family, order, and class"),
        )
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
        .arg(
            Arg::new("clust-id")
                .long("clust-id")
                .num_args(1)
                .default_value("0.95")
                .help("The min sequence identity for clustering"),
        )
        .arg(
            Arg::new("clust-cov")
                .long("clust-cov")
                .num_args(1)
                .default_value("0.95")
                .help("The min coverage of query and target for clustering"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Loading
    //----------------------------
    let outdir = args.get_one::<String>("outdir").unwrap();
    if outdir != "stdout" {
        fs::create_dir_all(outdir)?;
    }

    let mut ins = vec![];
    if args.contains_id("in") {
        for i in args.get_many::<String>("in").unwrap() {
            ins.push(i.to_string());
        }
    }

    let mut not_ins = vec![];
    if args.contains_id("not-in") {
        for i in args.get_many::<String>("not-in").unwrap() {
            not_ins.push(i.to_string());
        }
    }

    let mut ass_url_of = BTreeMap::new();
    let mut ass_species_of = BTreeMap::new();

    let mut bs_name_of = BTreeMap::new();
    let mut bs_species_of = BTreeMap::new();

    let mut mh_species_of = BTreeMap::new();
    let mut mh_level_of = BTreeMap::new();

    let mut count_species_of = BTreeMap::new();

    let mut pro_species_of = BTreeMap::new();

    if args.contains_id("infiles") {
        for infile in args.get_many::<String>("infiles").unwrap() {
            let reader = intspan::reader(infile);
            for line in reader.lines().map_while(Result::ok) {
                if line.starts_with('#') {
                    continue;
                }

                let fields: Vec<&str> = line.split('\t').collect();

                if fields.len() < 5 {
                    continue;
                }

                let name = fields[0];
                let url = fields[1];
                let sample = fields[2];

                // format species strings
                let species = fields[3];
                lazy_static! {
                    static ref RE_S1: Regex = Regex::new(r#"(?xi)\W+"#).unwrap();
                    static ref RE_S2: Regex = Regex::new(r#"(?xi)_+"#).unwrap();
                    static ref RE_S3: Regex = Regex::new(r#"(?xi)_$"#).unwrap();
                    static ref RE_S4: Regex = Regex::new(r#"(?xi)^_"#).unwrap();
                }
                let s1 = RE_S1.replace_all(species, "_");
                let s2 = RE_S2.replace_all(&s1, "_");
                let s3 = RE_S3.replace_all(&s2, "");
                let s4 = RE_S4.replace_all(&s3, "");
                let species_ = s4.to_string();

                let level = match fields[4] {
                    "Complete Genome" => "1",
                    "Chromosome" => "2",
                    "Scaffold" => "3",
                    "Contig" => "3",
                    _ => "5",
                };

                // ass
                // formatted species
                ass_url_of.insert(name.to_string(), url.to_string());
                ass_species_of.insert(name.to_string(), species_.to_string());

                // bs
                // formatted species
                if !sample.is_empty() {
                    bs_name_of.insert(sample.to_string(), name.to_string());
                    bs_species_of.insert(sample.to_string(), species_.to_string());
                }

                // mh
                // formatted species
                mh_species_of.insert(name.to_string(), species_.to_string());
                mh_level_of.insert(name.to_string(), level.to_string());

                // count
                // original species
                count_species_of.insert(name.to_string(), species.to_string());

                // pro
                // formatted species
                pro_species_of.insert(name.to_string(), species_.to_string());
            }
        }
    }

    let mut ranks = vec![];
    if args.contains_id("rank") {
        for rank in args.get_many::<String>("rank").unwrap() {
            ranks.push(rank.to_string());
        }
    }

    let mut lineages = vec![];
    if args.contains_id("lineage") {
        for rank in args.get_many::<String>("lineage").unwrap() {
            lineages.push(rank.to_string());
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
    context.insert("parallel", args.get_one::<String>("parallel").unwrap());

    context.insert("ass_url_of", &ass_url_of);
    context.insert("ass_species_of", &ass_species_of);

    context.insert("bs_name_of", &bs_name_of);
    context.insert("bs_species_of", &bs_species_of);

    context.insert("mh_species_of", &mh_species_of);
    context.insert("mh_level_of", &mh_level_of);
    context.insert("mh_sketch", args.get_one::<String>("sketch").unwrap());
    context.insert("mh_ani_ab", args.get_one::<String>("ani-ab").unwrap());
    context.insert("mh_ani_nr", args.get_one::<String>("ani-nr").unwrap());
    context.insert("mh_height", args.get_one::<String>("height").unwrap());

    context.insert("count_species_of", &count_species_of);
    context.insert("count_ranks", &ranks);
    context.insert("count_lineages", &lineages);
    context.insert("rank_col_of", &rank_col_of);

    context.insert("pro_species_of", &pro_species_of);
    context.insert("pro_clust_id", args.get_one::<String>("clust-id").unwrap());
    context.insert("pro_clust_cov", args.get_one::<String>("clust-cov").unwrap());

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
    if args.get_flag("ass") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/ASSEMBLY", outdir))?;
        }
        gen_ass_data(&context)?;
        gen_ass_rsync(&context)?;
        gen_ass_check(&context)?;
        gen_ass_reorder(&context)?;
        gen_ass_n50(&context)?;
        gen_ass_collect(&context)?;
        gen_ass_finish(&context)?;
    }

    if args.get_flag("bs") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/BioSample", outdir))?;
        }
        gen_bs_data(&context)?;
        gen_bs_download(&context)?;
        gen_bs_collect(&context)?;
    }

    if args.get_flag("mh") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/MinHash", outdir))?;
        }
        gen_mh_data(&context)?;
        gen_mh_compute(&context)?;
        gen_mh_species(&context)?;
        gen_mh_abnormal(&context)?;
        gen_mh_nr(&context)?;
        gen_mh_dist(&context)?;
    }

    if args.get_flag("count") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/Count", outdir))?;
        }
        gen_count_data(&context)?;
        gen_count_strains(&context)?;
        gen_count_rank(&context)?;
        gen_count_lineage(&context)?;
    }

    if args.get_flag("pro") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/Protein", outdir))?;
        }
        gen_pro_data(&context)?;
        gen_pro_collect(&context)?;
        gen_pro_info(&context)?;
        gen_pro_count(&context)?;
    }

    Ok(())
}

//----------------------------
// ASSEMBLY/url.tsv - name, url, species
//----------------------------
fn gen_ass_data(context: &Context) -> anyhow::Result<()> {
    let outname = "url.tsv";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let ass_url_of = context.get("ass_url_of").unwrap().as_object().unwrap();
    let ass_species_of = context.get("ass_species_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    for (key, value) in ass_url_of {
        let url = value.as_str().unwrap();
        let species = ass_species_of.get(key).unwrap().as_str().unwrap();

        // ftp   - ftp://ftp.ncbi.nlm.nih.gov/genomes/all/GCA/000/167/675/GCA_000167675.2_v2.0
        // rsync - ftp.ncbi.nlm.nih.gov::genomes/all/GCA/000/167/675/GCA_000167675.2_v2.0
        lazy_static! {
            static ref RE_URL: Regex =
                Regex::new(r#"(?xi)(ftp|https?)://ftp.ncbi.nlm.nih.gov/"#).unwrap();
        }
        let rsync = RE_URL.replace(url, "ftp.ncbi.nlm.nih.gov::");

        if url == rsync {
            eprintln!("Check the ftp url: [{}] {}", key, url);
        } else {
            writer.write_all(format!("{}\t{}\t{}\n", key, rsync, species).as_ref())?;
        }
    }

    Ok(())
}

//----------------------------
// ASSEMBLY/rsync.sh
//----------------------------
fn gen_ass_rsync(context: &Context) -> anyhow::Result<()> {
    let outname = "rsync.sh";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/ass_rsync.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// ASSEMBLY/check.sh
//----------------------------
fn gen_ass_check(context: &Context) -> anyhow::Result<()> {
    let outname = "check.sh";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/ass_check.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// ASSEMBLY/n50.sh
//----------------------------
fn gen_ass_n50(context: &Context) -> anyhow::Result<()> {
    let outname = "n50.sh";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/ass_n50.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// ASSEMBLY/collect.sh
//----------------------------
fn gen_ass_collect(context: &Context) -> anyhow::Result<()> {
    let outname = "collect.sh";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/ass_collect.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// ASSEMBLY/finish.sh
//----------------------------
fn gen_ass_finish(context: &Context) -> anyhow::Result<()> {
    let outname = "finish.sh";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/ass_finish.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// ASSEMBLY/clean.sh
//----------------------------
fn gen_ass_reorder(context: &Context) -> anyhow::Result<()> {
    let outname = "reorder.sh";
    eprintln!("Create ASSEMBLY/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/ASSEMBLY/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/ass_reorder.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// BioSample/sample.tsv - biosample, name, species
//----------------------------
fn gen_bs_data(context: &Context) -> anyhow::Result<()> {
    let outname = "sample.tsv";
    eprintln!("Create BioSample/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let bs_name_of = context.get("bs_name_of").unwrap().as_object().unwrap();
    let bs_species_of = context.get("bs_species_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/BioSample/{}", outdir, outname).as_ref())
    };

    for (key, value) in bs_name_of {
        let name = value.as_str().unwrap();
        let species = bs_species_of.get(key).unwrap().as_str().unwrap();

        writer.write_all(format!("{}\t{}\t{}\n", key, name, species).as_ref())?;
    }

    Ok(())
}

//----------------------------
// BioSample/download.sh
//----------------------------
fn gen_bs_download(context: &Context) -> anyhow::Result<()> {
    let outname = "download.sh";
    eprintln!("Create BioSample/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/BioSample/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/bs_download.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// BioSample/collect.sh
//----------------------------
fn gen_bs_collect(context: &Context) -> anyhow::Result<()> {
    let outname = "collect.sh";
    eprintln!("Create BioSample/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/BioSample/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/bs_collect.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// MinHash/species.tsv - name, species, level
//----------------------------
fn gen_mh_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let mh_species_of = context.get("mh_species_of").unwrap().as_object().unwrap();
    let mh_level_of = context.get("mh_level_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    for (key, value) in mh_species_of {
        let species = value.as_str().unwrap();
        let level = mh_level_of.get(key).unwrap().as_str().unwrap();

        writer.write_all(format!("{}\t{}\t{}\n", key, species, level).as_ref())?;
    }

    Ok(())
}

//----------------------------
// MinHash/compute.sh
//----------------------------
fn gen_mh_compute(context: &Context) -> anyhow::Result<()> {
    let outname = "compute.sh";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/mh_compute.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// MinHash/species.sh
//----------------------------
fn gen_mh_species(context: &Context) -> anyhow::Result<()> {
    let outname = "species.sh";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/mh_species.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// MinHash/abnormal.sh
//----------------------------
fn gen_mh_abnormal(context: &Context) -> anyhow::Result<()> {
    let outname = "abnormal.sh";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/mh_abnormal.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// MinHash/nr.sh
//----------------------------
fn gen_mh_nr(context: &Context) -> anyhow::Result<()> {
    let outname = "nr.sh";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/mh_nr.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// MinHash/dist.sh
//----------------------------
fn gen_mh_dist(context: &Context) -> anyhow::Result<()> {
    let outname = "dist.sh";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/mh_dist.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// Count/species.tsv - name, species
//----------------------------
fn gen_count_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create Count/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let count_species_of = context
        .get("count_species_of")
        .unwrap()
        .as_object()
        .unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Count/{}", outdir, outname).as_ref())
    };

    for (key, value) in count_species_of {
        let species = value.as_str().unwrap();

        writer.write_all(format!("{}\t{}\n", key, species).as_ref())?;
    }

    Ok(())
}

//----------------------------
// Count/strains.sh
//----------------------------
fn gen_count_strains(context: &Context) -> anyhow::Result<()> {
    let outname = "strains.sh";
    eprintln!("Create Count/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Count/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/count_strains.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// Count/rank.sh
//----------------------------
fn gen_count_rank(context: &Context) -> anyhow::Result<()> {
    let outname = "rank.sh";
    eprintln!("Create Count/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Count/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/count_rank.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// Count/lineage.sh
//----------------------------
fn gen_count_lineage(context: &Context) -> anyhow::Result<()> {
    let outname = "lineage.sh";
    eprintln!("Create Count/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Count/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/count_lineage.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// Protein/species.tsv - name, species
//----------------------------
fn gen_pro_data(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create Protein/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let species_of = context.get("pro_species_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Protein/{}", outdir, outname).as_ref())
    };

    for (key, value) in species_of {
        let species = value.as_str().unwrap();

        writer.write_all(format!("{}\t{}\n", key, species).as_ref())?;
    }

    Ok(())
}

//----------------------------
// Protein/collect.sh
//----------------------------
fn gen_pro_collect(context: &Context) -> anyhow::Result<()> {
    let outname = "collect.sh";
    eprintln!("Create Protein/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Protein/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/pro_collect.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// Protein/info.sh
//----------------------------
fn gen_pro_info(context: &Context) -> anyhow::Result<()> {
    let outname = "info.sh";
    eprintln!("Create Protein/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Protein/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/pro_info.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// Protein/count.sh
//----------------------------
fn gen_pro_count(context: &Context) -> anyhow::Result<()> {
    let outname = "count.sh";
    eprintln!("Create Protein/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/Protein/{}", outdir, outname).as_ref())
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("header", include_str!("../../templates/header.tera.sh")),
        ("t", include_str!("../../templates/pro_count.tera.sh")),
    ])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

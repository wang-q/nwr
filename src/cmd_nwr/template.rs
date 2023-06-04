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
    * And two Bash scripts
        * compute.sh
        * dist.sh
        * nr.sh

* --pro: PROTEIN/

"###,
        )
        .arg(
            Arg::new("infiles")
                .help(".assembly.tsv files")
                .required(true)
                .num_args(1..)
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
        // ASSEMBLY
        .arg(
            Arg::new("ass")
                .long("ass")
                .action(ArgAction::SetTrue)
                .help("Prepare ASSEMBLY materials"),
        )
        // BioSample
        .arg(
            Arg::new("bs")
                .long("bs")
                .action(ArgAction::SetTrue)
                .help("Prepare BioSample materials"),
        )
        // MinHash
        .arg(
            Arg::new("mh")
                .long("mh")
                .action(ArgAction::SetTrue)
                .help("Prepare MinHash materials"),
        )
        .arg(
            Arg::new("sketch")
                .long("sketch")
                .num_args(1)
                .default_value("100000")
                .help("Sketch size passed to `mash sketch`"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .num_args(1)
                .default_value("0.5")
                .help("Height value passed to `cutree()`"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Loading
    //----------------------------
    let mut ass_url_of = BTreeMap::new();
    let mut ass_species_of = BTreeMap::new();

    let mut bs_name_of = BTreeMap::new();
    let mut bs_species_of = BTreeMap::new();

    let mut mh_species_of = BTreeMap::new();

    let outdir = args.get_one::<String>("outdir").unwrap();
    if outdir != "stdout" {
        fs::create_dir_all(outdir)?;
    }

    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = intspan::reader(infile);
        for line in reader.lines().filter_map(|r| r.ok()) {
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
            let s2 = RE_S2.replace_all(&*s1, "_");
            let s3 = RE_S3.replace_all(&*s2, "");
            let s4 = RE_S4.replace_all(&*s3, "");
            let species_ = s4.to_string();

            // ass
            ass_url_of.insert(name.to_string(), url.to_string());
            ass_species_of.insert(name.to_string(), species_.to_string());

            // bs
            if !sample.is_empty() {
                bs_name_of.insert(sample.to_string(), name.to_string());
                bs_species_of.insert(sample.to_string(), species_.to_string());
            }

            // mh
            mh_species_of.insert(name.to_string(), species_.to_string());
        }
    }

    //----------------------------
    // Context
    //----------------------------
    let mut context = Context::new();

    context.insert("outdir", outdir);
    context.insert("ass_url_of", &ass_url_of);
    context.insert("ass_species_of", &ass_species_of);
    context.insert("bs_name_of", &bs_name_of);
    context.insert("bs_species_of", &bs_species_of);
    context.insert("mh_species_of", &mh_species_of);
    context.insert("mh_sketch", args.get_one::<String>("sketch").unwrap());
    context.insert("mh_height", args.get_one::<String>("height").unwrap());

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
        gen_ass_url(&context)?;
        gen_ass_rsync(&context)?;
        gen_ass_check(&context)?;
        gen_ass_n50(&context)?;
        gen_ass_collect(&context)?;
        gen_ass_finish(&context)?;
        gen_ass_reorder(&context)?;
    }

    if args.get_flag("bs") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/BioSample", outdir))?;
        }
        gen_bs_sample(&context)?;
        gen_bs_download(&context)?;
        gen_bs_collect(&context)?;
    }

    if args.get_flag("mh") {
        if outdir != "stdout" {
            fs::create_dir_all(format!("{}/MinHash", outdir))?;
        }
        gen_mh_species(&context)?;
        gen_mh_compute(&context)?;
        gen_mh_dist(&context)?;
    }

    Ok(())
}

//----------------------------
// rsync urls - name, url, species
//----------------------------
fn gen_ass_url(context: &Context) -> anyhow::Result<()> {
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

        if url == rsync.to_string() {
            eprintln!("Check the ftp url: [{}] {}", key, url);
        } else {
            writer.write_all(format!("{}\t{}\t{}\n", key, rsync, species).as_ref())?;
        }
    }

    Ok(())
}

//----------------------------
// rsync.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// check.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// n50.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// collect.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// finish.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// clean.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// sample.tsv - biosample, name, species
//----------------------------
fn gen_bs_sample(context: &Context) -> anyhow::Result<()> {
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
// download.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// collect.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// species.tsv - name, species
//----------------------------
fn gen_mh_species(context: &Context) -> anyhow::Result<()> {
    let outname = "species.tsv";
    eprintln!("Create MinHash/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let mh_species_of = context.get("mh_species_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/MinHash/{}", outdir, outname).as_ref())
    };

    for (key, value) in mh_species_of {
        let species = value.as_str().unwrap();

        writer.write_all(format!("{}\t{}\n", key, species).as_ref())?;
    }

    Ok(())
}

//----------------------------
// compute.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// dist.sh
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

    let rendered = tera.render("t", &context).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

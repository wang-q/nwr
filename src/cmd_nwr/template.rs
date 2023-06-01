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
        .about("Create dirs, files and scripts for a phylogenomic research")
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

* --ass - ASSEMBLY/
    * One TSV file:
        * url.tsv
    * And five Bash scripts:
        * rsync.sh
        * check.sh
        * collect.sh
        * n50.sh
        * finish.sh

* --bs - BioSample/
    * One TSV file:
        * sample.tsv
    * And two Bash scripts:
        * download.sh
        * collect.sh

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
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Loading
    //----------------------------
    let mut name_of = BTreeMap::new();
    let mut species_of = BTreeMap::new();

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
            let sample = fields[2];
            let species = fields[3];

            if !sample.is_empty() {
                name_of.insert(sample.to_string(), name.to_string());

                lazy_static! {
                    static ref RE1: Regex = Regex::new(r#"(?xi)\W+"#).unwrap();
                    static ref RE2: Regex = Regex::new(r#"(?xi)_+"#).unwrap();
                    static ref RE3: Regex = Regex::new(r#"(?xi)_$"#).unwrap();
                    static ref RE4: Regex = Regex::new(r#"(?xi)^_"#).unwrap();
                }
                let s1 = RE1.replace(species, "_");
                let s2 = RE2.replace(&*s1, "_");
                let s3 = RE3.replace(&*s2, "");
                let s4 = RE4.replace(&*s3, "");

                species_of.insert(sample.to_string(), s4.to_string());
            }
        }
    }

    //----------------------------
    // Context
    //----------------------------
    let mut context = Context::new();

    context.insert("outdir", outdir);
    context.insert("name_of", &name_of);
    context.insert("species_of", &species_of);

    //----------------------------
    // Writing
    //----------------------------
    if args.get_flag("bs") {
        fs::create_dir_all(format!("{}/BioSample", outdir))?;
        gen_bs_sample(&context)?;
        gen_bs_download(&context)?;
        gen_bs_collect(&context)?;
    }

    Ok(())
}

//----------------------------
// sample.tsv - biosample, name, species
//----------------------------
fn gen_bs_sample(context: &Context) -> anyhow::Result<()> {
    let outname = "sample.tsv";
    eprintln!("Create BioSample/{}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let name_of = context.get("name_of").unwrap().as_object().unwrap();
    let species_of = context.get("species_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/BioSample/{}", outdir, outname).as_ref())
    };

    for (key, value) in name_of {
        let name = value.as_str().unwrap();
        let species = species_of.get(key).unwrap().as_str().unwrap();

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

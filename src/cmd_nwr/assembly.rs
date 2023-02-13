use clap::*;
use intspan::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use tera::{Context, Tera};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("assembly")
        .about("Prepare ASSEMBLY materials")
        .after_help(
            r###"
One tsv file:
    * url.tsv

And three bash scripts:
    * rsync.sh
    * check.sh
    * collect.sh

will be generated.

"###,
        )
        .arg(
            Arg::new("infiles")
                .help("TSV files containing names and urls")
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
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Loading
    //----------------------------
    let mut url_of = BTreeMap::new();

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

            if fields.len() < 2 {
                continue;
            }

            let name = fields[0];
            let url = fields[1];

            url_of.insert(name.to_string(), url.to_string());
        }
    }

    //----------------------------
    // Context
    //----------------------------
    let mut context = Context::new();

    context.insert("outdir", outdir);
    context.insert("url_of", &url_of);

    let columns = vec![
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
    context.insert("columns", &columns);

    //----------------------------
    // Writing
    //----------------------------
    gen_url(&context)?;
    gen_rsync(&context)?;
    gen_check(&context)?;
    gen_collect(&context)?;

    Ok(())
}

//----------------------------
// rsync urls
//----------------------------
fn gen_url(context: &Context) -> anyhow::Result<()> {
    let outname = "url.tsv";
    eprintln!("Create {}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let url_of = context.get("url_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/{}", outdir, outname).as_ref())
    };

    for (key, value) in url_of {
        let url = value.as_str().unwrap();
        // ftp   - ftp://ftp.ncbi.nlm.nih.gov/genomes/all/GCA/000/167/675/GCA_000167675.2_v2.0
        // rsync - ftp.ncbi.nlm.nih.gov::genomes/all/GCA/000/167/675/GCA_000167675.2_v2.0
        lazy_static! {
            static ref RE1: Regex =
                Regex::new(r#"(?xi)(ftp|https?)://ftp.ncbi.nlm.nih.gov/"#).unwrap();
        }
        let rsync = RE1.replace(url, "ftp.ncbi.nlm.nih.gov::");

        if url == rsync.to_string() {
            eprintln!("Check the ftp url: [{}] {}", key, url);
        } else {
            writer.write_all(format!("{}\t{}\n", key, rsync).as_ref())?;
        }
    }

    Ok(())
}

//----------------------------
// rsync.sh
//----------------------------
fn gen_rsync(context: &Context) -> anyhow::Result<()> {
    let outname = "rsync.sh";
    eprintln!("Create {}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/{}", outdir, outname).as_ref())
    };

    // template
    let template = r###"#!/usr/bin/env bash

#----------------------------#
# helper functions
#----------------------------#
set +e

signaled () {
    echo >&2 Interrupted
    exit 1
}
trap signaled TERM QUIT INT

BASE_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
cd ${BASE_DIR}

#----------------------------#
# run
#----------------------------#
touch check.list

cat url.tsv |
    tsv-join -f check.list -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        echo >&2
        echo >&2 "==> {1}"
        mkdir -p {1}
        rsync -avP --no-links {2}/ {1}/ --exclude="assembly_status.txt"
    '

"###;

    let rendered = Tera::one_off(template, context, false).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// check.sh
//----------------------------
fn gen_check(context: &Context) -> anyhow::Result<()> {
    let outname = "check.sh";
    eprintln!("Create {}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/{}", outdir, outname).as_ref())
    };

    // template
    let template = r###"#!/usr/bin/env bash

#----------------------------#
# helper functions
#----------------------------#
set +e

signaled () {
    echo >&2 Interrupted
    exit 1
}
trap signaled TERM QUIT INT

BASE_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
cd ${BASE_DIR}

#----------------------------#
# run
#----------------------------#
touch check.list

cat check.list |
    tsv-uniq \
    > tmp.list
mv tmp.list check.list

cat url.tsv |
    tsv-join -f check.list -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [[ ! -e {1} ]]; then
            exit
        fi
        echo >&2 "==> {1}"
        cd {1}
        md5sum --check md5checksums.txt --quiet
        if [ "$?" -eq "0" ]; then
            echo "{1}" >> ../check.list
        fi
    '

"###;

    let rendered = Tera::one_off(template, context, false).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

//----------------------------
// collect.sh
//----------------------------
fn gen_collect(context: &Context) -> anyhow::Result<()> {
    let outname = "collect.sh";
    eprintln!("Create {}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/{}", outdir, outname).as_ref())
    };

    // template
    let template = r###"#!/usr/bin/env bash

#----------------------------#
# helper functions
#----------------------------#
set +e

signaled () {
    echo >&2 Interrupted
    exit 1
}
trap signaled TERM QUIT INT

BASE_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
cd ${BASE_DIR}

#----------------------------#
# run
#----------------------------#
touch check.list

echo "name,{{ columns | join(sep=",") }}" \
    > collect.csv

cat url.tsv |
    tsv-join -f check.list -k 1 |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        echo >&2
        echo >&2 "==> {1}"
        find {1} -type f -name "*_assembly_report.txt" |
            xargs cat |
            perl -nl -e '\''
                BEGIN { our %stat = (); }

                m{^#\s+} or next;
                s/^#\s+//;
                @O = split /\:\s*/;
                scalar @O == 2 or next;
                $O[0] =~ s/\s*$//g;
                $O[0] =~ s/\W/_/g;
                $O[1] =~ /([\w =.-]+)/ or next;
                $stat{$O[0]} = $1;

                END {
                    my @c;
                    for my $key ( qw( {{ columns | join(sep=" ") }} ) ) {
                        if (exists $stat{$key}) {
                            push @c, $stat{$key};
                        }
                        else {
                            push @c, q();
                        }
                    }
                    print join(q(,), q({1}), @c);
                }
            '\'' \
            >> collect.csv
    '

"###;

    let rendered = Tera::one_off(template, context, false).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}
use clap::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use tera::{Context, Tera};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("biosample")
        .about("Retrieve BioSample materials")
        .after_help(
            r###"
One tsv file:
    * sample.tsv

And two bash scripts:
    * download.sh
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

            if fields.len() < 2 {
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
    gen_sample(&context)?;
    gen_download(&context)?;
    gen_collect(&context)?;

    Ok(())
}

//----------------------------
// sample.tsv - biosample, name, species
//----------------------------
fn gen_sample(context: &Context) -> anyhow::Result<()> {
    let outname = "sample.tsv";
    eprintln!("Create {}", outname);

    let outdir = context.get("outdir").unwrap().as_str().unwrap();
    let name_of = context.get("name_of").unwrap().as_object().unwrap();
    let species_of = context.get("species_of").unwrap().as_object().unwrap();

    let mut writer = if outdir == "stdout" {
        intspan::writer("stdout")
    } else {
        intspan::writer(format!("{}/{}", outdir, outname).as_ref())
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
fn gen_download(context: &Context) -> anyhow::Result<()> {
    let outname = "download.sh";
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
# Helper functions
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
# Run
#----------------------------#
ulimit -n `ulimit -Hn`

cat sample.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        mkdir -p "{3}"
        if [ ! -s "{3}/{1}.txt" ]; then
            echo >&2 -e "==> {1}\t{2}\t{3}"
            curl -fsSL "https://www.ncbi.nlm.nih.gov/biosample/?term={1}&report=full&format=text" -o "{3}/{1}.txt"
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
# Helper functions
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
# Usage
#----------------------------#
USAGE="
Usage: $0 [COUNT_ATTR]

Default values:
    COUNT_ATTR  50

$ bash collect.sh 100

"

if ! [ -z "$1" ]; then
    if ! [[ $1 =~ ^[0-9]+$ ]]; then
        echo >&2 "$USAGE"
        exit 1
    fi
fi

COUNT_ATTR=${1:-50}

#----------------------------#
# Run
#----------------------------#
cat sample.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [ -s "{3}/{1}.txt" ]; then
            cat "{3}/{1}.txt" |
                perl -nl -e '\''
                    print $1 if m(\s+\/([\w_ ]+)=);
                '\''
        fi
    ' |
    tsv-uniq --at-least ${COUNT_ATTR} | # ignore rare attributes
    grep -v "^INSDC" |
    grep -v "^ENA" \
    > attributes.lst

# Headers
cat attributes.lst |
    (echo -e "#name\nBioSample" && cat) |
    tr '\n' '\t' |
    sed 's/\t$/\n/' \
    > biosample.tsv

cat sample.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        >&2 echo {1}

        cat "{3}/{1}.txt"  |
            perl -nl -MPath::Tiny -e '\''
                BEGIN {
                    our @keys = grep {/\S/} path(q{attributes.lst})->lines({chomp => 1});
                    our %stat = ();
                }

                m(\s+\/([\w_ ]+)=\"(.+)\") or next;
                my $k = $1;
                my $v = $2;
                if ( $v =~ m(\bNA|missing|Not applicable|not collected|not available|not provided|N\/A|not known|unknown\b)i ) {
                    $stat{$k} = q();
                } else {
                    $stat{$k} = $v;
                }

                END {
                    my @c;
                    for my $key ( @keys ) {
                        if (exists $stat{$key}) {
                            push @c, $stat{$key};
                        }
                        else {
                            push @c, q();
                        }
                    }
                    print join(qq(\t), q({2}), q({1}), @c);
                }
            '\''
    ' \
    >> biosample.tsv

"###;

    let rendered = Tera::one_off(template, context, false).unwrap();
    writer.write_all(rendered.as_ref())?;

    Ok(())
}

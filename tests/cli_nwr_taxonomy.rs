use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_info() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("--tsv")
        .arg("Viruses")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 3);
    assert!(stdout.contains("10239\tViruses"), "first record");

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 15);
    assert!(stdout.contains("ID: 10239"), "first record");

    Ok(())
}

#[test]
fn command_lineage() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert!(stdout.contains("Viruses\t10239"), "super kingdom");

    Ok(())
}

#[test]
fn command_restrict() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("2")
        .arg("-f")
        .arg("tests/nwr/taxon.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("Actinophage JHJ-1\t12347"), "virus");

    Ok(())
}

#[test]
fn command_restrict_e() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("2")
        .arg("-f")
        .arg("tests/nwr/taxon.tsv")
        .arg("-e")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("Actinophage JHJ-1\t12347"), "virus");

    Ok(())
}

#[test]
fn command_member() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("member")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Synechococcus phage S")
        .arg("-r")
        .arg("species")
        .arg("-r")
        .arg("no rank")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 36);
    assert!(stdout.contains("375032\tSynechococcus phage S"), "virus");

    Ok(())
}

#[test]
fn command_append() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg("tests/nwr/taxon-valid.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert_eq!(
        stdout.lines().next().unwrap(),
        "#sci_name\ttax_id\tsci_name"
    );
    assert!(
        stdout.contains("Actinophage JHJ-1\t12347\tActinophage JHJ-1"),
        "sci_name"
    );
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        3,
        "fields"
    );

    Ok(())
}

#[test]
fn command_common() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("common")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Actinophage JHJ-1")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((Actinophage JHJ-1[T=12347:rank=species],Bacillus phage bg1[T=483783:rank=species])unclassified bacterial viruses[T=12333:rank=no rank])root[T=1:rank=no rank];"));

    Ok(())
}

#[test]
fn command_append_rank() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg("-r")
        .arg("species")
        .arg("-r")
        .arg("family")
        .arg("--id")
        .arg("tests/nwr/taxon-valid.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert_eq!(
        stdout.lines().next().unwrap(),
        "#sci_name\ttax_id\tspecies\tspecies_id\tfamily\tfamily_id"
    );
    assert!(
        stdout.contains("\t12347\tActinophage JHJ-1\t12347"),
        "species"
    );
    assert!(stdout.contains("\tNA\t0"), "family");
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        6,
        "fields"
    );

    Ok(())
}

use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn command_invalid() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("foobar");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("recognized"));

    Ok(())
}

#[test]
fn command_txdb() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("txdb")
        .arg("--dir")
        .arg("tests/nwr/")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(std::path::Path::new("tests/nwr/taxonomy.sqlite").exists());
    assert_eq!(stdout.lines().count(), 8);

    Ok(())
}

#[test]
fn command_ardb() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ardb")
        .arg("--dir")
        .arg("tests/nwr/")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(std::path::Path::new("tests/nwr/ar_refseq.sqlite").exists());
    assert!(stdout.lines().count() > 10);

    Ok(())
}

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

#[test]
fn command_template_ass() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--ass")
        .arg("-o")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 7);
    assert!(stderr.contains("Create ASSEMBLY/url.tsv"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));

    Ok(())
}

#[test]
fn command_template_bs() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--bs")
        .arg("-o")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 3);
    assert!(stderr.contains("Create BioSample/sample.tsv"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));

    Ok(())
}

#[test]
fn command_template_mh() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--mh")
        .arg("--sketch")
        .arg("123456")
        .arg("-o")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 6);
    assert!(stderr.contains("Create MinHash/species.tsv"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));
    assert!(stdout.contains("123456"));

    Ok(())
}

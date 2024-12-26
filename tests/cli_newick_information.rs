use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_label() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 7);
    assert!(stdout.contains("Human\n"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-L")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 0);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-r")
        .arg("^ch")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-n")
        .arg("Homininae")
        .arg("-n")
        .arg("Pongo")
        .arg("-DM")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/catarrhini.comment.nwk")
        .arg("-c")
        .arg("species")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("\tHomo\n"));

    Ok(())
}

#[test]
fn command_stat() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("stat")
        .arg("tests/newick/hg38.7way.nwk")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(stdout.contains("leaf labels\t7"));

    Ok(())
}

#[test]
fn command_distance() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("root")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo\t60"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("parent")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo\t10"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("pairwise")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 100);
    assert!(stdout.contains("Homo\tPongo\t65"));
    assert!(stdout.contains("Pongo\tHomo\t65"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("lca")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 100);
    assert!(stdout.contains("Homo\tPongo\t35\t30"));
    assert!(stdout.contains("Homo\tHomo\t0\t0"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini_topo.nwk")
        .arg("-L")
        .arg("--mode")
        .arg("root")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(stdout.contains("Homininae\t3"));

    Ok(())
}

#[test]
fn command_distance_phylip() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("--mode")
        .arg("phylip")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 11);
    // two spaces after names
    // two spaces before distances
    assert!(stdout.contains("Homo    105  82"));

    Ok(())
}

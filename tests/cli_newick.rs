use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn command_indent() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("indent")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("--text")
        .arg(".   ")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 19);
    assert!(stdout.contains(".   .   Human:"));
    assert!(stdout.contains("\n.   Opossum:"));

    Ok(())
}

#[test]
fn command_order() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--nd")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("(C,(A,B));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--ndr")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((A,B),C);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--an")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((A,B),C);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--anr")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("(C,(B,A));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--anr")
        .arg("--ndr")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((B,A),C);"));

    Ok(())
}

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

    Ok(())
}

#[test]
fn command_rename() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("rename")
        .arg("tests/newick/abc.nwk")
        .arg("-n")
        .arg("C")
        .arg("-r")
        .arg("F")
        .arg("-l")
        .arg("A,B")
        .arg("-r")
        .arg("D")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((A,B)D,F);"));

    Ok(())
}

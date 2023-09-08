use assert_cmd::prelude::*; // Add methods on commands
use std::process::{Command, Stdio}; // Run programs

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
fn command_comment() -> anyhow::Result<()> {
    let mut cmd_color = Command::cargo_bin("nwr").unwrap()
        .arg("comment")
        .arg("tests/newick/abc.nwk")
        .arg("-n")
        .arg("A")
        .arg("-n")
        .arg("C")
        .arg("--color")
        .arg("green")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut cmd_dot = Command::cargo_bin("nwr").unwrap()
        .arg("comment")
        .arg("stdin")
        .arg("-l")
        .arg("A,B")
        .arg("--dot")
        .stdin(Stdio::from(cmd_color.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = cmd_dot.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().next().unwrap(), "((A[color=green],B)[dot=black],C[color=green]);");

    Ok(())
}
use assert_cmd::prelude::*;
use std::process::{Command, Stdio};

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
fn command_comment() -> anyhow::Result<()> {
    let cmd_color = Command::cargo_bin("nwr")
        .unwrap()
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
    let cmd_dot = Command::cargo_bin("nwr")
        .unwrap()
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

    assert_eq!(
        stdout.lines().next().unwrap(),
        "((A[color=green],B)[dot=black],C[color=green]);"
    );

    Ok(())
}

#[test]
fn command_comment_remove() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("comment")
        .arg("tests/newick/abc.comment.nwk")
        .arg("--remove")
        .arg("color=")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout.lines().next().unwrap(),
        "((A,B)[dot=black],C);"
    );

    Ok(())
}

#[test]
fn command_tex() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("tex")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("--bare")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 20);
    assert!(stdout.contains("\n  [,, tier=4\n"));
    assert!(stdout.contains("\n  [{Opossum},, tier=0]\n"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("tex")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("--bl")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 90);
    assert!(stdout.contains("\n  [,, l=40mm, l sep=0\n"));
    assert!(stdout
        .contains("\n  [{Opossum},, l=53mm, l sep=0, [{~},tier=0,edge={draw=none}]]\n"));

    Ok(())
}

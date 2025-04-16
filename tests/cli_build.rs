use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_build_upgma() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("build")
        .arg("upgma")
        .arg("tests/build/wiki.phy")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("a:8.5"));
    assert!(stdout.contains("c:14"));
    assert!(stdout.contains("):5.5"));

    Ok(())
}

#[test]
fn command_build_nj() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("build")
        .arg("nj")
        .arg("tests/build/wiki-nj.phy")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("a:2"));
    assert!(stdout.contains("c:4"));
    assert!(stdout.contains("d:2"));
    assert!(stdout.contains("):3"));

    Ok(())
}

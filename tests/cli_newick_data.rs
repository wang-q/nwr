use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_label() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("data")
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 7);
    assert!(stdout.contains("Human\n"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("data")
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-L")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 0);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("data")
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-r")
        .arg("^ch")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("data")
        .arg("label")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-n")
        .arg("Homininae")
        .arg("-n")
        .arg("Pongo")
        .arg("-DM")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 4);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("data")
        .arg("label")
        .arg("tests/newick/catarrhini.comment.nwk")
        .arg("-c")
        .arg("species")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("\tHomo\n"));

    Ok(())
}

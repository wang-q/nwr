use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn command_download_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("download").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Download"));

    Ok(())
}

#[test]
fn command_download_version() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("download").arg("--version");
    cmd.assert().success();

    Ok(())
}

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
    // Output lines may vary based on data size and progress indicators
    assert!(stdout.lines().count() >= 5);

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

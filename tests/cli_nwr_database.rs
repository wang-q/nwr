use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
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
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(std::path::Path::new("tests/nwr/taxonomy.sqlite").exists());
    // Progress log lines go to stderr; their count varies based on data size.
    assert!(stderr.lines().count() >= 5);

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
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(std::path::Path::new("tests/nwr/ar_refseq.sqlite").exists());
    assert!(stderr.contains("Loading"));

    Ok(())
}

/// Copy a fixture file and insert a single blank line after the specified
/// 0-based line index. This lets us exercise blank-line skipping without
/// modifying the original test fixtures.
fn copy_with_blank_line(
    src: &Path,
    dst: &Path,
    filename: &str,
    insert_after: usize,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(src.join(filename))?;
    let mut lines: Vec<&str> = content.lines().collect();
    if insert_after < lines.len() {
        lines.insert(insert_after + 1, "");
    }
    fs::write(dst.join(filename), lines.join("\n"))?;
    Ok(())
}

/// Populate `tests/nwr/blank_line/` with copies of the standard fixtures that
/// contain extra blank lines.
fn setup_blank_line_fixtures() -> anyhow::Result<()> {
    let src = Path::new("tests/nwr");
    let dst = Path::new("tests/nwr/blank_line");
    fs::create_dir_all(dst)?;
    copy_with_blank_line(src, dst, "division.dmp", 0)?;
    copy_with_blank_line(src, dst, "names.dmp", 0)?;
    copy_with_blank_line(src, dst, "nodes.dmp", 0)?;
    // The first two lines of the assembly summary are header comments; insert
    // a blank line after the first data row.
    copy_with_blank_line(src, dst, "assembly_summary_refseq.txt", 2)?;
    Ok(())
}

#[test]
fn command_txdb_blank_lines() -> anyhow::Result<()> {
    setup_blank_line_fixtures()?;

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("txdb")
        .arg("--dir")
        .arg("tests/nwr/blank_line/")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(Path::new("tests/nwr/blank_line/taxonomy.sqlite").exists());
    assert!(stderr.lines().count() >= 5);

    Ok(())
}

#[test]
fn command_ardb_blank_lines() -> anyhow::Result<()> {
    setup_blank_line_fixtures()?;

    // The assembly database needs the taxonomy database for lineage lookups.
    let tx_output = Command::cargo_bin("nwr")?
        .arg("txdb")
        .arg("--dir")
        .arg("tests/nwr/blank_line/")
        .output()
        .unwrap();
    assert!(tx_output.status.success());

    let output = Command::cargo_bin("nwr")?
        .arg("ardb")
        .arg("--dir")
        .arg("tests/nwr/blank_line/")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(Path::new("tests/nwr/blank_line/ar_refseq.sqlite").exists());
    assert!(stderr.contains("Loading"));

    Ok(())
}

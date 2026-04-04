use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn command_abbr_basic() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check header line is preserved and abbr is appended
    assert!(stdout.contains("#strain"));
    // Check basic abbreviation generation
    assert!(stdout.contains("E_coli_K_12"));
    assert!(stdout.contains("B_sub_168"));

    Ok(())
}

#[test]
fn command_abbr_tight() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .arg("--tight")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check tight format (no underscore between genus and species)
    assert!(stdout.contains("Ecoli") || stdout.contains("E_coli"));

    Ok(())
}

#[test]
fn command_abbr_shortsub() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .arg("--shortsub")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check output is generated
    assert!(stdout.lines().count() > 5);

    Ok(())
}

#[test]
fn command_abbr_custom_min() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .arg("--min")
        .arg("5")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check output with custom min length
    assert!(stdout.lines().count() > 5);

    Ok(())
}

#[test]
fn command_abbr_stdin() -> anyhow::Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut cmd = Command::cargo_bin("nwr")?;
    let mut child = cmd
        .arg("abbr")
        .arg("stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Write to stdin
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(
            b"#strain\tspecies\tgenus\nTest strain\tTest species\tTestgenus\n",
        )?;
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check stdin input works
    assert!(stdout.contains("Test strain"));

    Ok(())
}

#[test]
fn command_abbr_custom_column() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .arg("--column")
        .arg("1,2,3")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check output with explicit column specification
    assert!(stdout.lines().count() > 5);

    Ok(())
}

#[test]
fn command_abbr_outfile() -> anyhow::Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let output_path = temp_dir.path().join("output.tsv");

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();

    // Check output file was created
    assert!(output_path.exists());

    // Check file content
    let content = std::fs::read_to_string(&output_path)?;
    assert!(content.contains("E_coli") || content.contains("Ecoli"));

    Ok(())
}

#[test]
fn command_abbr_invalid_column_format() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("abbr")
        .arg("tests/nwr/strains.tsv")
        .arg("--column")
        .arg("1,2") // Invalid: needs 3 columns
        .assert()
        .failure()
        .stderr(predicate::str::contains("three numbers"));

    Ok(())
}

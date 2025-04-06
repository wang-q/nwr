use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_mat_pair() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("pair")
        .arg("tests/mat/IBPA.phy")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 55);
    assert!(stdout.contains("IBPA_ECOLI\tIBPA_ECOLI\t0\n"));
    assert!(stdout.contains("IBPA_ECOLI\tIBPA_ECOLI_GA\t0.058"));

    Ok(())
}

#[test]
fn command_mat_phylip() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("phylip")
        .arg("tests/mat/IBPA.fa.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 11);
    assert!(stdout.contains("IBPA_ECOLI\t0\t0.0669"));

    Ok(())
}

#[test]
fn command_mat_format_full() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("format")
        .arg("tests/mat/IBPA.phy")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 11);
    assert!(stdout.contains("IBPA_ECOLI\t0\t0.058394\t0.160584"));
    assert!(stdout.contains("IBPA_ECOLI_GA\t0.058394\t0\t0.10219"));

    Ok(())
}

#[test]
fn command_mat_format_lower() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("format")
        .arg("tests/mat/IBPA.phy")
        .arg("--mode")
        .arg("lower")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 11);
    assert!(stdout.contains("IBPA_ECOLI\n"));
    assert!(stdout.contains("IBPA_ECOLI_GA\t0.058394\n"));

    Ok(())
}

#[test]
fn command_mat_format_strict() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("format")
        .arg("tests/mat/IBPA.phy")
        .arg("--mode")
        .arg("strict")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 11);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "10"); // Number of sequences line

    // Check format of the first sequence
    let first_seq = lines[1];
    assert!(first_seq.starts_with("IBPA_ECOLI"));
    assert_eq!(first_seq.chars().take(10).count(), 10); // Name length limit
    assert!(first_seq.contains(" 0.000000")); // Formatted distance value

    Ok(())
}

#[test]
fn command_mat_subset() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("subset")
        .arg("tests/mat/IBPA.phy")
        .arg("tests/mat/IBPA.list")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    // Verify output
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0].trim(), "3"); // Number of sequences
    assert!(lines[1].starts_with("IBPA_ECOLI_GA\t0\t0.10219\t0.058394"));
    assert!(lines[3].starts_with("IBPA_ESCF3\t0.058394"));

    Ok(())
}

#[test]
fn command_mat_compare() -> anyhow::Result<()> {
    // Test single method
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("compare")
        .arg("tests/mat/IBPA.phy")
        .arg("tests/mat/IBPA.71.phy")
        .arg("--method")
        .arg("pearson")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    // Verify output format and approximate value
    assert!(stdout.contains("Method\tScore"));
    assert!(stdout.contains("pearson\t0.93"));

    // Test all methods
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("mat")
        .arg("compare")
        .arg("tests/mat/IBPA.phy")
        .arg("tests/mat/IBPA.71.phy")
        .arg("--method")
        .arg("all")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Verify matrix information in stderr
    assert!(stderr.contains("Sequences in matrices: 10 and 10"));
    assert!(stderr.contains("Common sequences: 10"));

    // Verify all methods are present with approximate values
    assert!(stdout.contains("pearson\t0.93"));
    assert!(stdout.contains("spearman\t0.91"));
    assert!(stdout.contains("mae\t0.11"));
    assert!(stdout.contains("cosine\t0.97"));
    assert!(stdout.contains("jaccard\t0.75"));
    assert!(stdout.contains("euclid\t1.22"));

    Ok(())
}

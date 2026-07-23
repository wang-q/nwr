use assert_cmd::prelude::*;
use std::io::Write;
use std::process::Command;

#[test]
fn command_info() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("--tsv")
        .arg("Viruses")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 3);
    assert!(stdout.contains("10239\tViruses"), "first record");

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 12);
    assert!(stdout.contains("ID: 10239"), "first record");

    Ok(())
}

#[test]
fn command_info_invalid_term() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("not_a_real_taxon_name")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such name"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("999999999")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such ID"));

    Ok(())
}

#[test]
fn command_info_duplicate_terms() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("--tsv")
        .arg("Viruses")
        .arg("Viruses")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let data_lines: Vec<&str> = stdout.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(
        data_lines.len(),
        2,
        "duplicate input terms produce duplicate output rows"
    );

    Ok(())
}

#[test]
fn command_info_sp_fallback() -> anyhow::Result<()> {
    // "Bacteriophage sp" (no dot) should resolve to "Bacteriophage sp." (tax_id 38018)
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Bacteriophage sp")
        .output()?;
    assert!(output.status.success(), "sp fallback should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("38018"));

    // Reverse direction: "Bacteriophage sp." (with dot) should still work (regression)
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("info")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Bacteriophage sp.")
        .output()?;
    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_lineage() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert!(stdout.contains("Viruses\t10239"), "super kingdom");

    Ok(())
}

#[test]
fn command_lineage_invalid_term() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("not_a_real_taxon_name")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such name"));

    Ok(())
}

#[test]
fn command_lineage_tax_id() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("12347") // Actinophage JHJ-1
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("12347"));

    Ok(())
}

#[test]
fn command_lineage_format() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
    }

    Ok(())
}

#[test]
fn command_lineage_root() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("root")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("1")); // Root tax_id

    Ok(())
}

#[test]
fn command_lineage_underscores() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("lineage")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Lactobacillus_phage_mv4")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("12392")); // Lactobacillus phage mv4

    Ok(())
}

#[test]
fn command_restrict() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("2")
        .arg("-f")
        .arg("tests/nwr/taxon.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("Actinophage JHJ-1\t12347"), "virus");

    Ok(())
}

#[test]
fn command_restrict_e() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("2")
        .arg("-f")
        .arg("tests/nwr/taxon.tsv")
        .arg("-e")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("Actinophage JHJ-1\t12347"), "virus");

    Ok(())
}

#[test]
fn command_member_invalid_term() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("member")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("not_a_real_taxon_name")
        .arg("-r")
        .arg("species")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such name"));

    Ok(())
}

#[test]
fn command_member() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("member")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Synechococcus phage S")
        .arg("-r")
        .arg("species")
        .arg("-r")
        .arg("no rank")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 36);
    assert!(stdout.contains("375032\tSynechococcus phage S"), "virus");

    Ok(())
}

#[test]
fn command_member_env() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("member")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("--env")
        .arg("Synechococcus phage S")
        .arg("-r")
        .arg("species")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(
        stdout.contains("375032\tSynechococcus phage S"),
        "ancestor present"
    );

    Ok(())
}

#[test]
fn command_append() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg("tests/nwr/taxon-valid.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert_eq!(
        stdout.lines().next().unwrap(),
        "#sci_name\ttax_id\tsci_name"
    );
    assert!(
        stdout.contains("Actinophage JHJ-1\t12347\tActinophage JHJ-1"),
        "sci_name"
    );
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        3,
        "fields"
    );

    Ok(())
}

#[test]
fn command_append_skips_empty_lines() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#sci_name\ttax_id")?;
    writeln!(temp)?;
    writeln!(temp, "Actinophage JHJ-1\t12347")?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg(path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "header plus valid line");

    Ok(())
}

#[test]
fn command_append_skips_whitespace_lines() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#sci_name\ttax_id")?;
    writeln!(temp, "   ")?;
    writeln!(temp, "\t")?;
    writeln!(temp, "Actinophage JHJ-1\t12347")?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg(path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "header plus valid line");

    Ok(())
}

#[test]
fn command_restrict_skips_empty_lines() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#name")?;
    writeln!(temp)?;
    writeln!(temp, "Actinophage JHJ-1")?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("1")
        .arg("-f")
        .arg(path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "header plus valid line");

    Ok(())
}

#[test]
fn command_restrict_skips_whitespace_lines() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#name")?;
    writeln!(temp, "   ")?;
    writeln!(temp, "\t")?;
    writeln!(temp, "Actinophage JHJ-1")?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("1")
        .arg("-f")
        .arg(path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "header plus valid line");

    Ok(())
}

#[test]
fn command_common_invalid_term() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("common")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("not_a_real_taxon_name")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such name"));

    Ok(())
}

#[test]
fn command_common() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("common")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Actinophage JHJ-1")
        .arg("Bacillus phage bg1")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains(
        "((Actinophage JHJ-1,Bacillus phage bg1)unclassified bacterial viruses)root;"
    ));

    Ok(())
}

#[test]
fn command_append_rank() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg("-r")
        .arg("species")
        .arg("-r")
        .arg("family")
        .arg("--id")
        .arg("tests/nwr/taxon-valid.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert_eq!(
        stdout.lines().next().unwrap(),
        "#sci_name\ttax_id\tspecies\tspecies_id\tfamily\tfamily_id"
    );
    assert!(
        stdout.contains("\t12347\tActinophage JHJ-1\t12347"),
        "species"
    );
    assert!(stdout.contains("\tNA\t0"), "family");
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        6,
        "fields"
    );

    Ok(())
}

#[test]
fn command_append_skips_invalid() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg("tests/nwr/taxon-invalid.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "only header and valid line");

    Ok(())
}

#[test]
fn command_append_strict_fails() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("append")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("-c")
        .arg("2")
        .arg("--strict")
        .arg("tests/nwr/taxon-invalid.tsv")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error converting term")
            || stderr.contains("Error getting taxon")
    );

    Ok(())
}

#[test]
fn command_restrict_skips_invalid() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("1")
        .arg("-f")
        .arg("tests/nwr/restrict-invalid.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2, "header plus valid line");

    Ok(())
}

#[test]
fn command_restrict_strict_fails() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("restrict")
        .arg("--dir")
        .arg("tests/nwr/")
        .arg("Viruses")
        .arg("-c")
        .arg("1")
        .arg("-f")
        .arg("tests/nwr/restrict-invalid.tsv")
        .arg("--strict")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error converting term"));

    Ok(())
}

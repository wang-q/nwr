use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn command_template_ass() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--ass")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 8);
    assert!(stderr.contains("Create ASSEMBLY/url.tsv"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));

    Ok(())
}

#[test]
fn command_template_bs() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--bs")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 14);
    assert!(stderr.contains("Create BioSample/sample.tsv"));
    assert!(stderr.contains("duplicate sample name"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));

    Ok(())
}

#[test]
fn command_template_mh() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--mh")
        .arg("--sketch")
        .arg("123456")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 6);
    assert!(stderr.contains("Create MinHash/species.tsv"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));
    assert!(stdout.contains("123456"));

    Ok(())
}

#[test]
fn command_template_count() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--count")
        .arg("--rank")
        .arg("genus")
        .arg("--rank")
        .arg("family")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("Count/"));
    assert!(stdout.lines().count() > 50);

    Ok(())
}

#[test]
fn command_template_pro() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--pro")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("Create Protein/"));
    assert!(stdout.lines().count() > 50);

    Ok(())
}

#[test]
fn command_template_invalid_rank() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--count")
        .arg("--rank")
        .arg("species")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid rank 'species'"));

    Ok(())
}

#[test]
fn command_template_skips_empty_lines() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#name\turl\tsample\tspecies\tassembly_level")?;
    writeln!(temp)?;
    writeln!(
        temp,
        "strain1\tftp://ftp.ncbi.nlm.nih.gov/genomes/all/foo\tsample1\tEscherichia coli\tComplete Genome"
    )?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg(path.to_str().unwrap())
        .arg("--ass")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_template_skips_whitespace_lines() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#name\turl\tsample\tspecies\tassembly_level")?;
    writeln!(temp, "   ")?;
    writeln!(temp, "\t")?;
    writeln!(
        temp,
        "strain1\tftp://ftp.ncbi.nlm.nih.gov/genomes/all/foo\tsample1\tEscherichia coli\tComplete Genome"
    )?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg(path.to_str().unwrap())
        .arg("--ass")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_kb_invalid() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("kb")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid document name"));

    Ok(())
}

#[test]
fn command_template_invalid_parallel_zero() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#name\turl\tsample\tspecies\tassembly_level")?;
    writeln!(
        temp,
        "strain1\tftp://ftp.ncbi.nlm.nih.gov/genomes/all/foo\tsample1\tEscherichia coli\tComplete Genome"
    )?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("template")
        .arg(path.to_str().unwrap())
        .arg("--ass")
        .arg("--parallel")
        .arg("0")
        .arg("--outdir")
        .arg("stdout")
        .assert()
        .failure()
        .stderr(predicate::str::contains("0"));

    Ok(())
}

#[test]
fn command_template_invalid_sketch_zero() -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    writeln!(temp, "#name\turl\tsample\tspecies\tassembly_level")?;
    writeln!(
        temp,
        "strain1\tftp://ftp.ncbi.nlm.nih.gov/genomes/all/foo\tsample1\tEscherichia coli\tComplete Genome"
    )?;
    let path = temp.into_temp_path();

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("template")
        .arg(path.to_str().unwrap())
        .arg("--mh")
        .arg("--sketch")
        .arg("0")
        .arg("--outdir")
        .arg("stdout")
        .assert()
        .failure()
        .stderr(predicate::str::contains("0"));

    Ok(())
}

#[test]
fn command_template_invalid_in_path() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--mh")
        .arg("--in")
        .arg("bad path;rm -rf /")
        .arg("--outdir")
        .arg("stdout")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid --in path"));

    Ok(())
}

#[test]
fn command_kb() -> anyhow::Result<()> {
    // Create a temporary directory for output
    let temp_dir = TempDir::new()?;

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("kb")
        .arg("bac120")
        .arg("--outdir")
        .arg(temp_dir.path())
        .output()
        .unwrap();

    // Check command succeeded
    assert!(output.status.success());

    // Check some files were extracted
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())?.collect();
    assert!(!entries.is_empty());

    // TempDir is automatically cleaned up when it goes out of scope
    Ok(())
}

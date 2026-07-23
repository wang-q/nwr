use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn command_seqdb_init() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Check command succeeded
    assert!(output.status.success());

    // Check database file was created
    let db_path = temp_dir.path().join("seq.sqlite");
    assert!(db_path.exists());

    Ok(())
}

#[test]
fn command_seqdb_load_strain() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // First init the database
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Then load strain data
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--strain")
        .arg("tests/nwr/seqdb_strains.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_load_strain_duplicate_ranks() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a strain file where multiple strains share the same rank.
    let strain_file = temp_dir.path().join("strains.tsv");
    std::fs::write(&strain_file, "strain_001\tspecies\nstrain_002\tspecies\n")?;

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--strain")
        .arg(strain_file)
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_load_size() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // First init the database
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Then load size data
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--size")
        .arg("tests/nwr/seqdb_sizes.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_load_clust() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // First init the database and load sizes (required for clust)
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--size")
        .arg("tests/nwr/seqdb_sizes.tsv")
        .output()
        .unwrap();

    // Then load cluster data
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--clust")
        .arg("tests/nwr/seqdb_clust.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_load_clust_duplicate_reps() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--size")
        .arg("tests/nwr/seqdb_sizes.tsv")
        .output()
        .unwrap();

    // Create a cluster file where the same rep maps to multiple seqs.
    let clust_file = temp_dir.path().join("rep_cluster.tsv");
    std::fs::write(&clust_file, "rep_001\tseq_001\nrep_001\tseq_002\n")?;

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--clust")
        .arg(clust_file)
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_load_anno() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // First init the database and load sizes (required for anno)
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--size")
        .arg("tests/nwr/seqdb_sizes.tsv")
        .output()
        .unwrap();

    // Then load annotation data
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--anno")
        .arg("tests/nwr/seqdb_anno.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_load_asmseq() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // First init the database and load strains and sizes
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Init failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--strain")
        .arg("tests/nwr/seqdb_strains.tsv")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Load strain failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--size")
        .arg("tests/nwr/seqdb_sizes.tsv")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Load size failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Then load asmseq data
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--asmseq")
        .arg("tests/nwr/seqdb_asmseq.tsv")
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("asmseq load stderr: {}", stderr);
    }
    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_default_paths() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Copy test files to temp dir with default names
    std::fs::copy(
        "tests/nwr/seqdb_strains.tsv",
        temp_dir.path().join("strains.tsv"),
    )?;
    std::fs::copy(
        "tests/nwr/seqdb_sizes.tsv",
        temp_dir.path().join("sizes.tsv"),
    )?;

    // First init the database
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Test with default paths (no explicit file arguments)
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--strain")
        .arg("--size")
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_invalid_rep_field() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a test file
    let test_file = temp_dir.path().join("test_rep.tsv");
    std::fs::write(&test_file, "family\trep\n")?;

    // First init the database
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Try to load rep with invalid field name
    let mut cmd = Command::cargo_bin("nwr")?;
    let rep_arg = format!("invalid_field={}", test_file.display());
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--rep")
        .arg(&rep_arg)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid rep field"));

    Ok(())
}

#[test]
fn command_seqdb_skips_empty_lines() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Create a strain file with blank lines interleaved.
    let strain_file = temp_dir.path().join("strains.tsv");
    std::fs::write(
        &strain_file,
        "strain_001\tspecies\n\nstrain_002\tspecies\n\n",
    )?;

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--strain")
        .arg(&strain_file)
        .output()
        .unwrap();

    assert!(output.status.success());

    Ok(())
}

#[test]
fn command_seqdb_full_workflow() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Init database
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--init")
        .output()
        .unwrap();

    // Load all data types in one command
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("seqdb")
        .arg("--workdir")
        .arg(temp_dir.path())
        .arg("--strain")
        .arg("tests/nwr/seqdb_strains.tsv")
        .arg("--size")
        .arg("tests/nwr/seqdb_sizes.tsv")
        .arg("--clust")
        .arg("tests/nwr/seqdb_clust.tsv")
        .arg("--anno")
        .arg("tests/nwr/seqdb_anno.tsv")
        .arg("--asmseq")
        .arg("tests/nwr/seqdb_asmseq.tsv")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify database file exists
    let db_path = temp_dir.path().join("seq.sqlite");
    assert!(db_path.exists());

    Ok(())
}

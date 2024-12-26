use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_template_ass() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("template")
        .arg("tests/assembly/Trichoderma.assembly.tsv")
        .arg("--ass")
        .arg("-o")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 7);
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
        .arg("-o")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 3);
    assert!(stderr.contains("Create BioSample/sample.tsv"));

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
        .arg("-o")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 5);
    assert!(stderr.contains("Create MinHash/species.tsv"));

    assert!(stdout.lines().count() > 100);
    assert!(stdout.contains("T_atrov"));
    assert!(stdout.contains("123456"));

    Ok(())
}

#[test]
fn command_kb() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd.arg("kb").arg("formats").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 10);
    assert!(stdout.contains("biosample "));

    Ok(())
}

use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_plot_venn2() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("venn")
        .arg("tests/plot/rocauc.result.tsv")
        .arg("tests/plot/mcox.05.result.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(-2.8, -1.8) { rocauc }"));
    assert!(stdout.contains("(-2,    0) { 669 }"));

    Ok(())
}

#[test]
fn command_plot_venn3() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("venn")
        .arg("tests/plot/rocauc.result.tsv")
        .arg("tests/plot/mcox.05.result.tsv")
        .arg("tests/plot/mcox.result.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(-2.8, -1.8) { rocauc }"));
    assert!(stdout.contains("(-2,   -0.2) { 161 }"));

    Ok(())
}

#[test]
fn command_plot_venn4() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("venn")
        .arg("tests/plot/rocauc.result.tsv")
        .arg("tests/plot/rocauc.result.tsv")
        .arg("tests/plot/mcox.05.result.tsv")
        .arg("tests/plot/mcox.result.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(-2.2, -2.6) { rocauc }"));
    assert!(stdout.contains("(-2.2,  1.5) { 161 }"));

    Ok(())
}

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

#[test]
fn command_plot_hh() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("hh")
        .arg("tests/plot/hist.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("31   0 0.0200"));
    assert!(stdout.contains("31   1 0.0000"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("hh")
        .arg("tests/plot/hist.tsv")
        .arg("-g")
        .arg("2")
        .arg("--bins")
        .arg("20")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("11   0 0.0600"));
    assert!(stdout.contains("11   1 0.1600"));

    Ok(())
}

#[test]
fn command_plot_nrps() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("nrps")
        .arg("tests/plot/srf.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(-0.4cm,0) -- (\\x1 + 0.2cm,0)"));
    assert!(!stdout.contains("\\textbf{M}ethyltransferase"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("plot")
        .arg("nrps")
        .arg("tests/plot/srf.tsv")
        .arg("--legend")
        .arg("--color")
        .arg("black")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("     draw=black,"));
    assert!(stdout.contains("\\textbf{M}ethyltransferase"));


    Ok(())
}

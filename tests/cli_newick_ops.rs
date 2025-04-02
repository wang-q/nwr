use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn command_order() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--nd")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(C,(A,B));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--ndr")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((A,B),C);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--an")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((A,B),C);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--anr")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(C,(B,A));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--anr")
        .arg("--ndr")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((B,A),C);"));

    Ok(())
}

#[test]
fn command_order_list() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("tests/newick/abcde.nwk")
        .arg("--list")
        .arg("tests/newick/abcde.list")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(C:1,(B:1,A:1)D:1)E;"));

    Ok(())
}

#[test]
fn command_order_species() -> anyhow::Result<()> {
    // Create a temporary directory for testing
    let tempdir = tempfile::tempdir()?;
    let temp_path = tempdir.path();

    std::fs::copy(
        "tests/newick/species.nwk",
        temp_path.join("species.nwk"),
    )?;

    // Generate a list of labels from the tree
    let mut cmd = Command::cargo_bin("nwr")?;
    cmd.arg("data")
        .arg("label")
        .arg("species.nwk")
        .arg("-o")
        .arg("species.list")
        .current_dir(temp_path)
        .output()?;

    // Order the tree using the generated list
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("species.nwk")
        .arg("--list")
        .arg("species.list")
        .current_dir(temp_path)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    // Compare the ordered tree with the original one
    // They should be identical as the list was generated from the original order
    let original = std::fs::read_to_string("tests/newick/species.nwk")?;
    assert_eq!(stdout.trim(), original.trim());

    // gene tree
    std::fs::copy(
        "tests/newick/pmxc.nwk",
        temp_path.join("pmxc.nwk"),
    )?;

    // Order pmxc.nwk using the generated list
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("order")
        .arg("pmxc.nwk")
        .arg("--list")
        .arg("species.list")
        .current_dir(temp_path)
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    // Read the original pmxc.nwk file
    let original = std::fs::read_to_string("tests/newick/pmxc.nwk")?;

    // The ordered tree should be different from the original one
    assert_ne!(stdout.trim(), original.trim());

    Ok(())
}

#[test]
fn command_rename() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("rename")
        .arg("tests/newick/abc.nwk")
        .arg("-n")
        .arg("C")
        .arg("-r")
        .arg("F")
        .arg("-l")
        .arg("A,B")
        .arg("-r")
        .arg("D")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((A,B)D,F);"));

    Ok(())
}

#[test]
fn command_replace() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("replace")
        .arg("tests/newick/abc.nwk")
        .arg("tests/newick/abc.replace.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((Homo,Pan),Gorilla);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("replace")
        .arg("tests/newick/abc.nwk")
        .arg("tests/newick/abc.replace.tsv")
        .arg("--mode")
        .arg("species")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((A[S=Homo],B[S=Pan]),C[S=Gorilla]);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("replace")
        .arg("tests/newick/abc.nwk")
        .arg("tests/newick/abc3.replace.tsv")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((Homo[color=red],Pan[color=red]),Gorilla[color=red]);"));

    Ok(())
}

#[test]
fn command_topo() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("topo")
        .arg("tests/newick/catarrhini.nwk")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((((Gorilla,(Pan,Homo)"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("topo")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-IL")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((((,(,)),),),(((,),),(,)));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("topo")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-IL")
        .arg("--bl")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("((((:16,(:10,:10)"));

    Ok(())
}

#[test]
fn command_subtree() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("subtree")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-n")
        .arg("Human")
        .arg("-n")
        .arg("Rhesus")
        .arg("-M")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 0);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("subtree")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-n")
        .arg("Human")
        .arg("-n")
        .arg("Rhesus")
        .arg("-r")
        .arg("^ch")
        .arg("-M")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((Human:0.007,Chimp:0.00684):0.027,Rhesus:0.037601):0.11;"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("subtree")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-n")
        .arg("Human")
        .arg("-n")
        .arg("Rhesus")
        .arg("-r")
        .arg("^ch")
        .arg("-M")
        .arg("-c")
        .arg("Primates")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("Primates:0.11[member=3:tri=white]"));

    Ok(())
}

#[test]
fn command_subtree_taxon() -> anyhow::Result<()> {
    let path = dirs::home_dir().unwrap().join(".nwr/");
    if !path.exists() {
        return Ok(());
    }

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("subtree")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-t")
        .arg("Hominidae")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains(
        "((Gorilla:16,(Pan:10,Homo:10):10)Homininae:15,Pongo:30)Hominidae:15;"
    ));

    Ok(())
}

#[test]
fn command_prune() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("prune")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-n")
        .arg("Homo")
        .arg("-n")
        .arg("Pan")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(!stdout.contains("Homo:10"));
    assert!(!stdout.contains("Gorilla:16"));
    assert!(stdout.contains("Gorilla:31"));

    Ok(())
}

#[test]
fn command_reroot() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("reroot")
        .arg("tests/newick/catarrhini_wrong.nwk")
        .arg("-n")
        .arg("Cebus")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(Cebus,(((Cercopithecus,(Macaca,Papio)),Simias),(Hylobates,(Pongo,(Gorilla,(Pan,Homo))))));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("reroot")
        .arg("tests/newick/abcde.nwk")
        .arg("-n")
        .arg("B")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(B:0.5,(A:1,C:2)D:0.5);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("ops")
        .arg("reroot")
        .arg("tests/newick/bs.nw")
        .arg("-n")
        .arg("C")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("(C,(B,(A,(((D,E)86,F)93,(G,(H,I))100)100)41)61);"));

    Ok(())
}

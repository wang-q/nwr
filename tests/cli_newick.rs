use assert_cmd::prelude::*; // Add methods on commands
use std::process::{Command, Stdio}; // Run programs

#[test]
fn command_label() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 7);
    assert!(stdout.contains("Human\n"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-L")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 0);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-r")
        .arg("^ch")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-n")
        .arg("Homininae")
        .arg("-n")
        .arg("Pongo")
        .arg("-DM")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("label")
        .arg("tests/newick/catarrhini.comment.nwk")
        .arg("-c")
        .arg("species")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("\tHomo\n"));

    Ok(())
}

#[test]
fn command_stat() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("stat")
        .arg("tests/newick/hg38.7way.nwk")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(stdout.contains("leaf labels\t7"));

    Ok(())
}

#[test]
fn command_distance() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("root")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo\t60"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("parent")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 10);
    assert!(stdout.contains("Homo\t10"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("pairwise")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 100);
    assert!(stdout.contains("Homo\tPongo\t65"));
    assert!(stdout.contains("Pongo\tHomo\t65"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-I")
        .arg("--mode")
        .arg("lca")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 100);
    assert!(stdout.contains("Homo\tPongo\t35\t30"));
    assert!(stdout.contains("Homo\tHomo\t0\t0"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini_topo.nwk")
        .arg("-L")
        .arg("--mode")
        .arg("root")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 5);
    assert!(stdout.contains("Homininae\t3"));

    Ok(())
}

#[test]
fn command_distance_phylip() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("distance")
        .arg("tests/newick/catarrhini.nwk")
        .arg("--mode")
        .arg("phylip")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 11);
    // two spaces after names
    // two spaces before distances
    assert!(stdout.contains("Homo    105  82"));

    Ok(())
}

#[test]
fn command_order() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--nd")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("(C,(A,B));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--ndr")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((A,B),C);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--an")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((A,B),C);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--anr")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("(C,(B,A));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("order")
        .arg("tests/newick/abc.nwk")
        .arg("--anr")
        .arg("--ndr")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((B,A),C);"));

    Ok(())
}

#[test]
fn command_rename() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
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
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((A,B)D,F);"));

    Ok(())
}

#[test]
fn command_replace() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("replace")
        .arg("tests/newick/abc.nwk")
        .arg("tests/newick/abc.replace.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((Homo,Pan),Gorilla);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("replace")
        .arg("tests/newick/abc.nwk")
        .arg("tests/newick/abc.replace.tsv")
        .arg("--mode")
        .arg("species")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((A[S=Homo],B[S=Pan]),C[S=Gorilla]);"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("replace")
        .arg("tests/newick/abc.nwk")
        .arg("tests/newick/abc3.replace.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((Homo[color=red],Pan[color=red]),Gorilla[color=red]);"));

    Ok(())
}

#[test]
fn command_topo() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("topo")
        .arg("tests/newick/catarrhini.nwk")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((((Gorilla,(Pan,Homo)"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("topo")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-IL")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((((,(,)),),),(((,),),(,)));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("topo")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-IL")
        .arg("--bl")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("((((:16,(:10,:10)"));

    Ok(())
}

#[test]
fn command_subtree() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("subtree")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-n")
        .arg("Human")
        .arg("-n")
        .arg("Rhesus")
        .arg("-M")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 0);

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("subtree")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("-n")
        .arg("Human")
        .arg("-n")
        .arg("Rhesus")
        .arg("-r")
        .arg("^ch")
        .arg("-M")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("((Human:0.007,Chimp:0.00684):0.027,Rhesus:0.037601):0.11;"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
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
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("Primates:0.11[member=3]"));

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
        .arg("subtree")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-t")
        .arg("Hominidae")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

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
        .arg("prune")
        .arg("tests/newick/catarrhini.nwk")
        .arg("-n")
        .arg("Homo")
        .arg("-n")
        .arg("Pan")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(!stdout.contains("Homo:10"));
    assert!(!stdout.contains("Gorilla:16"));
    assert!(stdout.contains("Gorilla:31"));

    Ok(())
}

#[test]
fn command_reroot() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("reroot")
        .arg("tests/newick/catarrhini_wrong.nwk")
        .arg("-n")
        .arg("Cebus")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("(Cebus,(((Cercopithecus,(Macaca,Papio)),Simias),(Hylobates,(Pongo,(Gorilla,(Pan,Homo))))));"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("reroot")
        .arg("tests/newick/abcde.nwk")
        .arg("-n")
        .arg("B")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("(B:0.5,(A:1,C:2)D:0.5);"));

    Ok(())
}

#[test]
fn command_indent() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("indent")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("--text")
        .arg(".   ")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 19);
    assert!(stdout.contains(".   .   Human:"));
    assert!(stdout.contains("\n.   Opossum:"));

    Ok(())
}

#[test]
fn command_comment() -> anyhow::Result<()> {
    let cmd_color = Command::cargo_bin("nwr")
        .unwrap()
        .arg("comment")
        .arg("tests/newick/abc.nwk")
        .arg("-n")
        .arg("A")
        .arg("-n")
        .arg("C")
        .arg("--color")
        .arg("green")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let cmd_dot = Command::cargo_bin("nwr")
        .unwrap()
        .arg("comment")
        .arg("stdin")
        .arg("-l")
        .arg("A,B")
        .arg("--dot")
        .stdin(Stdio::from(cmd_color.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = cmd_dot.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        stdout.lines().next().unwrap(),
        "((A[color=green],B)[dot=black],C[color=green]);"
    );

    Ok(())
}

#[test]
fn command_tex() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("tex")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("--bare")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 20);
    assert!(stdout.contains("\n  [,, tier=4\n"));
    assert!(stdout.contains("\n  [{Opossum},, tier=0]\n"));

    let mut cmd = Command::cargo_bin("nwr")?;
    let output = cmd
        .arg("tex")
        .arg("tests/newick/hg38.7way.nwk")
        .arg("--bl")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 90);
    assert!(stdout.contains("\n  [,, l=40mm, l sep=0\n"));
    assert!(stdout
        .contains("\n  [{Opossum},, l=53mm, l sep=0, [{~},tier=0,edge={draw=none}]]\n"));

    Ok(())
}

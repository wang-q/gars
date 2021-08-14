use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs // f32

#[test]
fn command_range() -> Result<(), Box<dyn std::error::Error>> {
    // drop
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // pos
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("pos")
        .arg("tests/S288c/spo11_hot.pos.txt")
        .arg("tests/S288c/spo11_hot.pos.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("71 positions in total"));
    assert!(stdout.contains("142 positions in total"));

    Ok(())
}

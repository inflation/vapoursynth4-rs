use std::{path::Path, process::Command};

use testresult::TestResult;

#[test]
fn run_vspipe() -> TestResult {
    let cmd_path = Path::new(if cfg!(target_os = "windows") {
        "../vapoursynth/msvc_project/x64/Debug/VSPipe.exe"
    } else {
        "../vapoursynth/build/bin/vspipe"
    })
    .canonicalize()?;

    let mut cmd = Command::new(cmd_path);
    cmd.args(["-i", "test.vpy"]);
    let output = cmd.output()?;
    panic!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());

    Ok(())
}

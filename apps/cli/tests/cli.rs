use serde_json::Value;
use std::{path::PathBuf, process::Command};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_klab"))
}
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}
fn json(command: &mut Command) -> Value {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn imports_detects_and_lists() {
    let temp = tempfile::tempdir().unwrap();
    let mut cli = cli();
    cli.current_dir(temp.path()).arg("--json");
    let mut import = cli;
    import
        .args(["workspace", "import"])
        .arg(fixture().join("kbuild"));
    let report = json(&mut import);
    assert_eq!(report["adapter"], "kbuild");
    assert_eq!(report["version"], "6.1.0");
    assert_eq!(
        json(
            crate::cli()
                .current_dir(temp.path())
                .args(["workspace", "list"])
        )[0],
        report["root"]
    );
}

#[test]
fn compares_abi_metadata() {
    let root = fixture().join("synthetic");
    let report = json(
        cli()
            .args(["abi", "compare"])
            .arg(root.join("base.symvers"))
            .arg(root.join("candidate.symvers")),
    );
    assert_eq!(report["state"], "potentially_incompatible");
    assert_eq!(report["changed"][0]["after"]["crc"], 2);
}

#[cfg(target_os = "linux")]
#[test]
fn compiles_and_inspects_real_fixture() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture().join("kbuild");
    let build = || {
        let mut cmd = cli();
        cmd.current_dir(temp.path())
            .args(["--json", "build", "--root"])
            .arg(&root)
            .arg("--profile")
            .arg(root.join("profile.json"));
        cmd
    };
    let refusal = build().output().unwrap();
    assert!(!refusal.status.success());
    assert!(!temp.path().join("out").exists());
    assert_eq!(json(build().arg("--dry-run"))["program"], "make");
    let report = json(build().arg("--trust"));
    assert_eq!(report["build"]["success"], true);
    assert_eq!(
        report["build"]["artifacts"][0]["sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert!(
        json(
            cli()
                .args(["elf", "inspect"])
                .arg(temp.path().join("out/vmlinux"))
        )["sections"]
            .is_array()
    );
    assert_eq!(
        json(cli().current_dir(temp.path()).args(["build", "history"]))[0]["report"],
        report
    );
}

#[cfg(target_os = "linux")]
#[test]
fn checks_module_imports_and_release() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture().join("synthetic");
    let module = temp.path().join("fixture.ko");
    assert!(
        Command::new("cc")
            .arg("-c")
            .arg(root.join("module.c"))
            .arg("-o")
            .arg(&module)
            .status()
            .unwrap()
            .success()
    );
    let metadata = json(cli().args(["module", "inspect"]).arg(&module));
    assert_eq!(metadata["metadata"]["license"][0], "MIT");
    assert_eq!(metadata["imports"][0], "fixture_init");
    let compatible = json(
        cli()
            .args(["module", "compare"])
            .arg(&module)
            .arg(root.join("base.symvers"))
            .args(["--release", "6.1.0"]),
    );
    assert_eq!(
        compatible["state"],
        "compatible_according_to_available_checks"
    );
    let mismatch = json(
        cli()
            .args(["module", "compare"])
            .arg(&module)
            .arg(root.join("base.symvers"))
            .args(["--release", "6.2.0"]),
    );
    assert_eq!(mismatch["state"], "potentially_incompatible");
}

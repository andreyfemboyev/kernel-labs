use klab_core::{Result, invalid, output};
use std::{path::Path, process::Command};

fn git(root: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(root).args([
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "core.fsmonitor=false",
        "--no-pager",
    ]);
    cmd
}
pub fn status(root: &Path) -> Result<String> {
    output(git(root).args(["status", "--porcelain=v1", "--branch"]))
}
pub fn commit(root: &Path) -> Result<String> {
    output(git(root).args(["rev-parse", "--verify", "HEAD"])).map(|s| s.trim().into())
}
pub fn history(root: &Path) -> Result<String> {
    output(git(root).args(["log", "-n", "50", "--format=%h %s"]))
}
pub fn diff(root: &Path) -> Result<String> {
    output(git(root).args(["diff", "--no-ext-diff", "--no-textconv", "HEAD", "--"]))
}

pub fn patch(root: &Path, file: &Path, apply: bool, approved: bool) -> Result<String> {
    if apply && !approved {
        return Err(invalid("patch application requires --approve"));
    }
    let mut cmd = git(root);
    cmd.arg("apply");
    if !apply {
        cmd.arg("--check");
    }
    output(cmd.arg("--").arg(file.canonicalize()?))
}

pub fn clone(url: &str, destination: &Path, approved: bool) -> Result<String> {
    if !approved {
        return Err(invalid(
            "cloning downloads sources; review URL and destination and pass --approve",
        ));
    }
    if !url.starts_with("https://") {
        return Err(invalid("clone supports HTTPS URLs"));
    }
    output(
        Command::new("git")
            .args(["-c", "core.hooksPath=/dev/null", "clone", "--"])
            .args([url])
            .arg(destination),
    )
}

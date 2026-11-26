use serde::Serialize;
use std::{env, path::PathBuf};

#[derive(Serialize)]
pub struct Capability {
    pub program: &'static str,
    pub path: Option<PathBuf>,
}

pub fn detect() -> Vec<Capability> {
    [
        "git",
        "repo",
        "make",
        "bash",
        "bazel",
        "clang",
        "gcc",
        "ld.lld",
        "llvm-objdump",
        "llvm-symbolizer",
        "dtc",
        "fdtoverlay",
        "dt-validate",
        "avbtool",
        "stgdiff",
        "adb",
        "fastboot",
        "qemu-system-aarch64",
        "qemu-system-x86_64",
        "launch_cvd",
        "emulator",
        "wsl.exe",
    ]
    .into_iter()
    .map(|program| Capability {
        program,
        path: find(program),
    })
    .collect()
}

pub fn find(program: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .filter_map(|dir| {
            let path = dir.join(program);
            let metadata = path.metadata().ok()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 == 0 {
                    return None;
                }
            }
            metadata.is_file().then_some(path)
        })
        .next()
}

use klab_core::{Result, invalid};
use std::{
    path::Path,
    process::{Child, Command},
};

pub fn available() -> Vec<klab_toolchain::Capability> {
    klab_toolchain::detect()
        .into_iter()
        .filter(|cap| {
            matches!(
                cap.program,
                "qemu-system-aarch64" | "qemu-system-x86_64" | "launch_cvd" | "emulator"
            )
        })
        .collect()
}

pub fn qemu_arm64(kernel: &Path, initrd: &Path, approved: bool) -> Result<Child> {
    if !approved {
        return Err(invalid("explicit approval required to launch the emulator"));
    }
    Ok(Command::new("qemu-system-aarch64")
        .args([
            "-machine",
            "virt",
            "-cpu",
            "cortex-a57",
            "-m",
            "1024",
            "-smp",
            "2",
            "-nographic",
            "-nic",
            "none",
            "-no-reboot",
            "-kernel",
        ])
        .arg(kernel.canonicalize()?)
        .arg("-initrd")
        .arg(initrd.canonicalize()?)
        .args(["-append", "console=ttyAMA0"])
        .spawn()?)
}

use klab_core::{Result, invalid, read_text};
use serde::{Deserialize, Serialize};
use std::{
    path::{Component, Path},
    process::Command,
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub protocol: u32,
    pub name: String,
    pub executable: String,
    pub capabilities: Vec<String>,
}

pub fn inspect(path: &Path) -> Result<Manifest> {
    let manifest: Manifest =
        serde_json::from_str(&read_text(path)?).map_err(|e| invalid(e.to_string()))?;
    if manifest.protocol != 1
        || manifest.capabilities.as_slice() != ["export"]
        || !matches!(
            Path::new(&manifest.executable)
                .components()
                .collect::<Vec<_>>()
                .as_slice(),
            [Component::Normal(_)]
        )
    {
        return Err(invalid(
            "protocol 1 requires an export plugin and an executable basename",
        ));
    }
    Ok(manifest)
}

pub fn export(manifest: &Path, report: &Path, approved: bool) -> Result<String> {
    if !approved {
        return Err(invalid("plugin execution requires --approve"));
    }
    let info = inspect(manifest)?;
    let root = manifest.canonicalize()?;
    let root = root
        .parent()
        .ok_or_else(|| invalid("missing plugin directory"))?;
    let program = root.join(info.executable).canonicalize()?;
    if !program.starts_with(root) {
        return Err(invalid("plugin executable escapes manifest directory"));
    }
    klab_core::output(
        Command::new(program)
            .env_clear()
            .arg("export-v1")
            .arg(report.canonicalize()?),
    )
}

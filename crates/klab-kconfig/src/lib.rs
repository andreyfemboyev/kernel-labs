use klab_core::{Result, invalid, output, read_text};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

pub type Config = BTreeMap<String, String>;

pub fn parse(text: &str) -> Result<Config> {
    let mut symbols = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        let pair = if let Some(key) = line
            .strip_prefix("# CONFIG_")
            .and_then(|s| s.strip_suffix(" is not set"))
        {
            Some((key, "n"))
        } else if let Some(value) = line.strip_prefix("CONFIG_") {
            Some(
                value
                    .split_once('=')
                    .ok_or_else(|| invalid(format!("invalid assignment: {line}")))?,
            )
        } else {
            None
        };
        if let Some((key, value)) = pair {
            if key.is_empty()
                || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || value.is_empty()
            {
                return Err(invalid(format!("invalid assignment: {line}")));
            }
            symbols.insert(format!("CONFIG_{key}"), value.into());
        }
    }
    Ok(symbols)
}

#[derive(Serialize)]
pub struct Change {
    symbol: String,
    before: Option<String>,
    after: Option<String>,
}

pub fn diff(before: &Config, after: &Config) -> Vec<Change> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|key| {
            let a = before.get(key);
            let b = after.get(key);
            (a != b).then(|| Change {
                symbol: key.clone(),
                before: a.cloned(),
                after: b.cloned(),
            })
        })
        .collect()
}

pub fn resolve(root: &Path, config: &Path, arch: &str, trusted: bool) -> Result<Config> {
    if !trusted {
        return Err(invalid(
            "Kconfig resolution executes workspace Makefiles; pass --trust after review",
        ));
    }
    if !matches!(arch, "arm" | "arm64" | "x86" | "x86_64") {
        return Err(invalid("unsupported Kconfig architecture"));
    }
    let stage = tempfile::tempdir()?;
    let config_copy = stage.path().join(".config");
    std::fs::write(&config_copy, klab_core::read(config, 64 * 1024 * 1024)?)?;
    output(
        Command::new("make")
            .current_dir(root)
            .arg(format!("O={}", stage.path().display()))
            .arg(format!("ARCH={arch}"))
            .arg("olddefconfig")
            .env("KCONFIG_CONFIG", &config_copy),
    )?;
    parse(&read_text(&config_copy)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_strings_and_unset() {
        let config = parse("# CONFIG_FOO is not set\nCONFIG_BAR=\"a=b\"\n").unwrap();
        assert_eq!(config["CONFIG_FOO"], "n");
        assert_eq!(config["CONFIG_BAR"], "\"a=b\"");
        assert_eq!(diff(&config, &Config::new()).len(), 2);
    }
}

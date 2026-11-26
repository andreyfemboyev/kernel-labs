use klab_core::{Result, invalid};
use klab_elf::{Object, ObjectSection, bytes, parse, symbols};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Serialize)]
pub struct Module {
    pub metadata: BTreeMap<String, Vec<String>>,
    pub imports: Vec<String>,
    pub defined_global_symbols: Vec<String>,
    pub kernel_signature_marker: bool,
}

pub fn inspect(path: &Path) -> Result<Module> {
    let bytes = bytes(path)?;
    let elf = parse(&bytes)?;
    let section = elf
        .section_by_name(".modinfo")
        .ok_or_else(|| invalid("ELF has no .modinfo section"))?;
    let mut metadata: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for field in section
        .data()
        .map_err(|e| invalid(e.to_string()))?
        .split(|&v| v == 0)
    {
        if let Some((key, value)) = String::from_utf8_lossy(field).split_once('=') {
            metadata.entry(key.into()).or_default().push(value.into());
        }
    }
    let symbols = symbols(&elf);
    Ok(Module {
        metadata,
        imports: symbols
            .iter()
            .filter(|s| s.undefined && s.global)
            .map(|s| s.name.clone())
            .collect(),
        defined_global_symbols: symbols
            .into_iter()
            .filter(|s| !s.undefined && s.global)
            .map(|s| s.name)
            .collect(),
        kernel_signature_marker: bytes.ends_with(b"~Module signature appended~\n"),
    })
}

pub fn compatibility(
    module: &Path,
    symvers: &Path,
    expected_release: Option<&str>,
) -> Result<serde_json::Value> {
    let module = inspect(module)?;
    let exports = klab_abi::parse(&klab_core::read_text(symvers)?)?;
    let missing: Vec<_> = module
        .imports
        .iter()
        .filter(|name| !exports.contains_key(*name))
        .collect();
    let vermagic = module
        .metadata
        .get("vermagic")
        .and_then(|values| values.first());
    let release = vermagic.and_then(|value| value.split_whitespace().next());
    let release_matches = expected_release
        .zip(release)
        .map(|(expected, actual)| expected == actual);
    let state = if release_matches == Some(false) || (!exports.is_empty() && !missing.is_empty()) {
        "potentially_incompatible"
    } else if exports.is_empty() || release_matches.is_none() {
        "insufficient_information"
    } else {
        "compatible_according_to_available_checks"
    };
    Ok(serde_json::json!({
        "state": state,
        "expected_release": expected_release,
        "vermagic": vermagic,
        "release_matches": release_matches,
        "imports_absent_from_symvers": missing,
        "scope": "Import names and release token only. CRCs, signatures, export eligibility and runtime compatibility are unverified."
    }))
}

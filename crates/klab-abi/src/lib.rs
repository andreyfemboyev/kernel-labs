use klab_core::{Result, invalid, read_text};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Symbol {
    crc: u32,
    module: String,
    export: String,
    namespace: String,
}

pub fn parse(text: &str) -> Result<BTreeMap<String, Symbol>> {
    let mut symbols = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        if !(4..=5).contains(&fields.len()) {
            return Err(invalid(format!("invalid Module.symvers row {}", i + 1)));
        }
        let crc = u32::from_str_radix(fields[0].trim_start_matches("0x"), 16)
            .map_err(|_| invalid(format!("invalid CRC on row {}", i + 1)))?;
        let symbol = Symbol {
            crc,
            module: fields[2].into(),
            export: fields[3].into(),
            namespace: fields.get(4).unwrap_or(&"").to_string(),
        };
        if symbols.insert(fields[1].into(), symbol).is_some() {
            return Err(invalid(format!("duplicate symbol {}", fields[1])));
        }
    }
    Ok(symbols)
}

pub fn compare(base: &Path, candidate: &Path) -> Result<serde_json::Value> {
    let base = parse(&read_text(base)?)?;
    let candidate = parse(&read_text(candidate)?)?;
    let removed: Vec<_> = base
        .keys()
        .filter(|k| !candidate.contains_key(*k))
        .collect();
    let added: Vec<_> = candidate
        .keys()
        .filter(|k| !base.contains_key(*k))
        .collect();
    let changed: Vec<_> = base
        .iter()
        .filter_map(|(name, before)| {
            let after = candidate.get(name)?;
            (after != before)
                .then(|| serde_json::json!({"symbol":name,"before":before,"after":after}))
        })
        .collect();
    let state = if base.is_empty() || candidate.is_empty() {
        "insufficient_information"
    } else if !removed.is_empty() || !changed.is_empty() {
        "potentially_incompatible"
    } else {
        "compatible_according_to_available_checks"
    };
    Ok(
        serde_json::json!({"state":state,"added":added,"removed":removed,"changed":changed,
        "scope":"Module.symvers metadata only. Runtime compatibility requires additional evidence."}),
    )
}

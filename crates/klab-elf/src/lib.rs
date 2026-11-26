use klab_core::{Result, invalid, output, read};
pub use object::{Object, ObjectSection, ObjectSymbol};
use serde::Serialize;
use std::{path::Path, process::Command};

pub fn bytes(path: &Path) -> Result<Vec<u8>> {
    read(path, 512 * 1024 * 1024)
}

pub fn parse(bytes: &[u8]) -> Result<object::File<'_>> {
    if !bytes.starts_with(b"\x7fELF") {
        return Err(invalid("expected an ELF file"));
    }
    object::File::parse(bytes).map_err(|e| invalid(e.to_string()))
}

#[derive(Serialize)]
pub struct Section {
    name: String,
    address: u64,
    size: u64,
    relocations: usize,
}

#[derive(Serialize)]
pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub undefined: bool,
    pub global: bool,
}

pub fn symbols(elf: &object::File<'_>) -> Vec<Symbol> {
    elf.symbols()
        .chain(elf.dynamic_symbols())
        .filter_map(|s| {
            let name = s.name().ok()?;
            if name.is_empty() {
                return None;
            }
            Some(Symbol {
                name: name.into(),
                address: s.address(),
                size: s.size(),
                undefined: s.is_undefined(),
                global: s.is_global(),
            })
        })
        .collect()
}

pub fn inspect(path: &Path) -> Result<serde_json::Value> {
    let bytes = bytes(path)?;
    let elf = parse(&bytes)?;
    let sections: Vec<_> = elf
        .sections()
        .map(|s| Section {
            name: s.name().unwrap_or("<invalid UTF-8>").into(),
            address: s.address(),
            size: s.size(),
            relocations: s.relocations().count(),
        })
        .collect();
    let build_id = elf
        .build_id()
        .map_err(|e| invalid(e.to_string()))?
        .map(|b| b.iter().map(|v| format!("{v:02x}")).collect::<String>());
    Ok(serde_json::json!({
        "architecture":format!("{:?}",elf.architecture()),"kind":format!("{:?}",elf.kind()),
        "bits":if elf.is_64() {64} else {32}, "little_endian":elf.is_little_endian(),
        "entry":elf.entry(),"bytes":bytes.len(),"build_id":build_id,
        "btf":elf.section_by_name(".BTF").is_some(),"sections":sections
    }))
}

pub fn symbolize(path: &Path, address: u64) -> Result<String> {
    let path = path.canonicalize()?;
    output(
        Command::new("llvm-symbolizer")
            .arg("--obj")
            .arg(path)
            .arg(format!("0x{address:x}")),
    )
}

pub fn disassemble(path: &Path) -> Result<String> {
    output(
        Command::new("llvm-objdump")
            .args(["-d", "--"])
            .arg(path.canonicalize()?),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_malformed() {
        for data in [b"abc".as_slice(), b"\x7fELF\x02\x01"] {
            assert!(parse(data).is_err());
        }
    }
}

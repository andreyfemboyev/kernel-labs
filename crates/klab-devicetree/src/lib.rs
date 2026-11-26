use klab_core::{Result, bounded_slice, invalid, output, u32_at};
use serde::Serialize;
use std::{path::Path, process::Command};

#[derive(Debug, Serialize)]
pub struct Node {
    path: String,
    properties: Vec<Property>,
}

#[derive(Debug, Serialize)]
pub struct Property {
    name: String,
    hex: String,
    strings: Option<Vec<String>>,
}

fn cstr(bytes: &[u8], at: usize) -> Result<(&str, usize)> {
    let rest = bytes
        .get(at..)
        .ok_or_else(|| invalid("invalid string offset"))?;
    let len = rest
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| invalid("unterminated DTB string"))?;
    Ok((
        std::str::from_utf8(&rest[..len]).map_err(|e| invalid(e.to_string()))?,
        at + len + 1,
    ))
}

pub fn parse(bytes: &[u8]) -> Result<Vec<Node>> {
    let word = |offset| u32_at(bytes, offset, true).map(|v| v as usize);
    if word(0)? != 0xd00dfeed {
        return Err(invalid("not a flattened device tree"));
    }
    let total = word(4)?;
    let struct_off = word(8)?;
    let strings_off = word(12)?;
    let reserve_off = word(16)?;
    if word(20)? != 17 || word(24)? > 17 {
        return Err(invalid("DTB version 17 required"));
    }
    let strings_size = word(32)?;
    let struct_size = word(36)?;
    let bytes = bounded_slice(bytes, 0, total)?;
    if struct_off < 40
        || strings_off < 40
        || reserve_off < 40
        || struct_off % 4 != 0
        || reserve_off % 8 != 0
    {
        return Err(invalid("invalid DTB section offsets"));
    }
    let structure = bounded_slice(bytes, struct_off, struct_size)?;
    bounded_slice(bytes, strings_off, strings_size)?;
    if reserve_off >= struct_off || struct_off + struct_size > strings_off {
        return Err(invalid("overlapping or out-of-order DTB sections"));
    }
    let reservation_map = &bytes[reserve_off..struct_off];
    if !reservation_map
        .as_chunks::<16>()
        .0
        .iter()
        .any(|entry| entry == &[0; 16])
    {
        return Err(invalid("unterminated DTB reservation map"));
    }
    let strings = bounded_slice(bytes, strings_off, strings_size)?;
    let mut nodes: Vec<Node> = Vec::new();
    let mut stack = Vec::<usize>::new();
    let mut pos = 0;
    let mut expanded = 0usize;
    loop {
        let token = u32_at(structure, pos, true)?;
        pos += 4;
        match token {
            1 => {
                let (name, end) = cstr(structure, pos)?;
                pos = (end + 3) & !3;
                if stack.len() >= 128
                    || nodes.len() >= 100_000
                    || name.contains('/')
                    || name.len() > 255
                {
                    return Err(invalid("DTB node limits exceeded"));
                }
                let path = if let Some(&parent) = stack.last() {
                    if name.is_empty() {
                        return Err(invalid("empty child node name"));
                    }
                    format!("{}/{name}", nodes[parent].path.trim_end_matches('/'))
                } else if nodes.is_empty() && name.is_empty() {
                    "/".into()
                } else {
                    return Err(invalid("invalid DTB root"));
                };
                expanded += path.len();
                if path.len() > 4096 || expanded > 64 * 1024 * 1024 {
                    return Err(invalid("DTB path too long"));
                }
                stack.push(nodes.len());
                nodes.push(Node {
                    path,
                    properties: Vec::new(),
                });
            }
            2 => {
                stack.pop().ok_or_else(|| invalid("unexpected END_NODE"))?;
            }
            3 => {
                let node = *stack
                    .last()
                    .ok_or_else(|| invalid("property outside a node"))?;
                let size = u32_at(structure, pos, true)? as usize;
                let offset = u32_at(structure, pos + 4, true)? as usize;
                pos += 8;
                let data = bounded_slice(structure, pos, size)?;
                let (name, _) = cstr(strings, offset)?;
                if name.is_empty() || name.len() > 255 || nodes[node].properties.len() >= 4096 {
                    return Err(invalid("invalid property or property limit exceeded"));
                }
                expanded += name.len() + data.len() * 2;
                if expanded > 64 * 1024 * 1024 {
                    return Err(invalid("DTB decoded output exceeds the size limit"));
                }
                let strings = data
                    .strip_suffix(&[0])
                    .filter(|d| d.len() <= 16384)
                    .and_then(|d| {
                        if d.split(|&b| b == 0).take(257).count() <= 256
                            && d.iter()
                                .all(|&b| b == 0 || b.is_ascii_graphic() || b == b' ')
                        {
                            Some(
                                d.split(|&b| b == 0)
                                    .map(|s| String::from_utf8_lossy(s).into_owned())
                                    .collect(),
                            )
                        } else {
                            None
                        }
                    });
                nodes[node].properties.push(Property {
                    name: name.into(),
                    hex: hex::encode(data),
                    strings,
                });
                pos += (size + 3) & !3;
            }
            4 => {}
            9 if stack.is_empty() && !nodes.is_empty() => return Ok(nodes),
            _ => return Err(invalid("unexpected DTB token")),
        }
    }
}

pub fn convert(path: &Path, from_binary: bool, includes: &[std::path::PathBuf]) -> Result<()> {
    let path = path.canonicalize()?;
    let mut command = Command::new("dtc");
    let (input, kind, ext) = if from_binary {
        ("dtb", "dts", "dts")
    } else {
        ("dts", "dtb", "dtb")
    };
    // Capture through a temporary file because compiled blobs are binary.
    let stage = tempfile::tempdir()?;
    let temp = stage.path().join(ext);
    command.args(["-I", input, "-O", kind, "-o"]).arg(&temp);
    for dir in includes {
        command.arg("-i").arg(dir.canonicalize()?);
    }
    output(command.arg(&path))?;
    let destination = path.with_extension(format!("klab.{ext}"));
    let mut file = std::fs::File::create_new(destination)?;
    std::io::copy(&mut std::fs::File::open(temp)?, &mut file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimal_tree() {
        let bytes: Vec<_> = [
            0xd00dfeedu32,
            72,
            56,
            72,
            40,
            17,
            16,
            0,
            0,
            16,
            0,
            0,
            0,
            0,
            1,
            0,
            2,
            9,
        ]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
        assert_eq!(parse(&bytes).unwrap()[0].path, "/");
        for prefix in 0..bytes.len() {
            assert!(parse(&bytes[..prefix]).is_err());
        }
    }
}

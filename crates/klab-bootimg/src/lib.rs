use klab_core::{Result, bounded_slice, invalid, u32_at};
use serde::Serialize;
use std::{io::Write, path::Path};

#[derive(Debug, Serialize)]
pub struct Component {
    name: &'static str,
    offset: usize,
    bytes: usize,
    compression: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Image {
    format: &'static str,
    version: u32,
    page_size: usize,
    components: Vec<Component>,
    avb_footer: bool,
}

fn align(size: usize, page: usize) -> Result<usize> {
    size.checked_add(page - 1)
        .map(|v| v & !(page - 1))
        .ok_or_else(|| invalid("image alignment overflow"))
}

fn compression(data: &[u8]) -> &'static str {
    if data.starts_with(b"\x1f\x8b") {
        "gzip"
    } else if data.starts_with(b"\x04\x22\x4d\x18") || data.starts_with(b"\x02\x21\x4c\x18") {
        "lz4"
    } else if data.starts_with(b"\x28\xb5\x2f\xfd") {
        "zstd"
    } else if data.starts_with(b"\xfd7zXZ\0") {
        "xz"
    } else if data.starts_with(b"070701") || data.starts_with(b"070702") {
        "cpio"
    } else {
        "unknown_or_uncompressed"
    }
}

pub fn parse(bytes: &[u8]) -> Result<Image> {
    let word = |offset| u32_at(bytes, offset, false);
    let size = |offset| word(offset).map(|v| v as usize);
    let (format, version, page, header, parts) = if bytes.starts_with(b"ANDROID!") {
        let version = word(40)?;
        match version {
            0..=2 => {
                let header = [1632, 1648, 1660][version as usize];
                if version >= 1 && size(1644)? != header {
                    return Err(invalid("unexpected legacy Android boot header size"));
                }
                let mut parts = vec![
                    ("kernel", size(8)?),
                    ("ramdisk", size(16)?),
                    ("second", size(24)?),
                ];
                if version >= 1 {
                    parts.push(("recovery_dtbo", size(1632)?));
                }
                if version == 2 {
                    parts.push(("dtb", size(1648)?));
                }
                ("android_boot", version, size(36)?, header, parts)
            }
            3 | 4 => {
                let header = if version == 3 { 1580 } else { 1584 };
                if size(20)? != header {
                    return Err(invalid("unexpected Android boot header size"));
                }
                let mut parts = vec![("kernel", size(8)?), ("ramdisk", size(12)?)];
                if version == 4 {
                    parts.push(("boot_signature", size(1580)?));
                }
                ("android_boot", version, 4096, header, parts)
            }
            _ => {
                return Err(invalid(format!(
                    "unsupported Android boot header version {version}"
                )));
            }
        }
    } else if bytes.starts_with(b"VNDRBOOT") {
        let version = word(8)?;
        let header = match version {
            3 => 2112,
            4 => 2128,
            _ => {
                return Err(invalid(format!(
                    "unsupported vendor boot header version {version}"
                )));
            }
        };
        if size(2096)? != header {
            return Err(invalid("unexpected vendor boot header size"));
        }
        let mut parts = vec![("vendor_ramdisk", size(24)?), ("dtb", size(2100)?)];
        if version == 4 {
            parts.extend([
                ("vendor_ramdisk_table", size(2112)?),
                ("bootconfig", size(2124)?),
            ]);
        }
        ("vendor_boot", version, size(12)?, header, parts)
    } else {
        return Err(invalid("unrecognized Android boot image magic"));
    };
    if !page.is_power_of_two() || !(512..=65536).contains(&page) {
        return Err(invalid("invalid image page size"));
    }
    bounded_slice(bytes, 0, header)?;
    let mut offset = align(header, page)?;
    let mut components = Vec::new();
    for (name, size) in parts {
        if size != 0 {
            if name == "recovery_dtbo" {
                let declared =
                    u64::from_le_bytes(bounded_slice(bytes, 1636, 8)?.try_into().unwrap());
                if declared != offset as u64 {
                    return Err(invalid(
                        "recovery DTBO offset differs from boot image layout",
                    ));
                }
            }
            let data = bounded_slice(bytes, offset, size)?;
            components.push(Component {
                name,
                offset,
                bytes: size,
                compression: compression(data),
            });
        }
        offset = offset
            .checked_add(align(size, page)?)
            .ok_or_else(|| invalid("image offset overflow"))?;
    }
    let avb_footer = bytes.len() >= 64 && bytes[bytes.len() - 64..].starts_with(b"AVBf");
    Ok(Image {
        format,
        version,
        page_size: page,
        components,
        avb_footer,
    })
}

pub fn extract(bytes: &[u8], name: &str, destination: &Path) -> Result<()> {
    let image = parse(bytes)?;
    let component = image
        .components
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| invalid("component absent or unsupported"))?;
    let data = bounded_slice(bytes, component.offset, component.bytes)?;
    std::fs::File::create_new(destination)?.write_all(data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_modern_and_rejects_truncated() {
        let mut bytes = vec![0; 4099];
        bytes[..8].copy_from_slice(b"ANDROID!");
        for (offset, value) in [(8, 3u32), (20, 1580), (40, 3)] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        assert_eq!(parse(&bytes).unwrap().components[0].offset, 4096);
        assert!(parse(&bytes[..4098]).is_err());
    }

    #[test]
    fn extracts_component_without_overwriting() {
        let stage = tempfile::tempdir().unwrap();
        let mut bytes = vec![0; 4099];
        bytes[..8].copy_from_slice(b"ANDROID!");
        for (offset, value) in [(8, 3u32), (20, 1580), (40, 3)] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[4096..].copy_from_slice(b"abc");
        let output = stage.path().join("kernel");
        assert!(extract(&bytes, "../kernel", &output).is_err());
        assert!(!output.exists());
        extract(&bytes, "kernel", &output).unwrap();
        assert!(extract(&bytes, "kernel", &output).is_err());
        assert_eq!(std::fs::read(output).unwrap(), b"abc");
    }
}

use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path, process::Command};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Invalid(String),
    #[error("{0} exited with {1}: {2}")]
    Process(String, std::process::ExitStatus, String),
}

pub fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid(format!("{} exceeds {limit} bytes", path.display())));
    }
    Ok(bytes)
}

pub fn read_text(path: &Path) -> Result<String> {
    String::from_utf8(read(path, 64 * 1024 * 1024)?).map_err(|e| invalid(e.to_string()))
}

pub fn sha256(path: &Path) -> Result<String> {
    let mut digest = Sha256::new();
    std::io::copy(&mut File::open(path)?, &mut digest)?;
    Ok(format!("{:x}", digest.finalize()))
}

pub fn output(command: &mut Command) -> Result<String> {
    let name = command.get_program().to_string_lossy().into_owned();
    let result = command.output()?;
    if !result.status.success() {
        return Err(Error::Process(
            name,
            result.status,
            String::from_utf8_lossy(&result.stderr).into(),
        ));
    }
    Ok(String::from_utf8_lossy(&result.stdout).into())
}

pub fn bounded_slice(bytes: &[u8], offset: usize, size: usize) -> Result<&[u8]> {
    offset
        .checked_add(size)
        .and_then(|end| bytes.get(offset..end))
        .ok_or_else(|| invalid("truncated or out-of-bounds input"))
}

pub fn u32_at(bytes: &[u8], offset: usize, big_endian: bool) -> Result<u32> {
    let array = bounded_slice(bytes, offset, 4)?.try_into().unwrap();
    Ok(if big_endian {
        u32::from_be_bytes(array)
    } else {
        u32::from_le_bytes(array)
    })
}

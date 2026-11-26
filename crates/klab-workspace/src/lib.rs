use klab_core::{Result, invalid, read_text};
use rusqlite::{Connection, params};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Adapter {
    Kleaf,
    Legacy,
    Kbuild,
    Vendor,
}

#[derive(Debug, Serialize)]
pub struct Workspace {
    pub root: PathBuf,
    pub adapter: Adapter,
    pub version: Option<String>,
    pub architectures: Vec<String>,
}

pub fn inspect(path: &Path) -> Result<Workspace> {
    let root = path.canonicalize()?;
    let makefile = root.join("Makefile");
    let has = |name: &str| root.join(name).is_file();
    let adapter = if has("tools/bazel") || has("build/kernel/kleaf/bazel.py") {
        Adapter::Kleaf
    } else if has("build/build.sh") || has("build.sh") {
        Adapter::Legacy
    } else if has("Makefile") && has("Kconfig") {
        Adapter::Kbuild
    } else {
        return Err(invalid(
            "expected a Kleaf, legacy Android, or Kbuild source root",
        ));
    };
    let version = if makefile.is_file() {
        let text = read_text(&makefile)?;
        ["VERSION", "PATCHLEVEL", "SUBLEVEL"]
            .iter()
            .map(|key| {
                text.lines()
                    .filter_map(|line| line.split_once('='))
                    .find_map(|(k, v)| {
                        let value = v.trim();
                        (k.trim() == *key
                            && !value.is_empty()
                            && value.bytes().all(|b| b.is_ascii_digit()))
                        .then(|| value.to_owned())
                    })
            })
            .collect::<Option<Vec<_>>>()
            .map(|values| values.join("."))
    } else {
        None
    };
    let architectures = ["arm", "arm64", "x86"]
        .into_iter()
        .filter(|arch| root.join("arch").join(arch).is_dir())
        .map(String::from)
        .collect();
    Ok(Workspace {
        root,
        adapter,
        version,
        architectures,
    })
}

pub struct Store(Connection);

fn db_error(error: rusqlite::Error) -> klab_core::Error {
    invalid(error.to_string())
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Connection::open(path).map_err(db_error)?;
        db.busy_timeout(Duration::from_secs(5)).map_err(db_error)?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS workspace (root TEXT PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS build (id INTEGER PRIMARY KEY,
                 created TEXT DEFAULT CURRENT_TIMESTAMP, report TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS audit (id INTEGER PRIMARY KEY,
                 created TEXT DEFAULT CURRENT_TIMESTAMP, operation TEXT NOT NULL, detail TEXT NOT NULL);"
        ).map_err(db_error)?;
        Ok(Self(db))
    }
    pub fn import(&self, path: &Path) -> Result<Workspace> {
        let workspace = inspect(path)?;
        let root = workspace
            .root
            .to_str()
            .ok_or_else(|| invalid("workspace path must be UTF-8"))?;
        self.0
            .execute("INSERT OR IGNORE INTO workspace VALUES (?1)", [root])
            .map_err(db_error)?;
        Ok(workspace)
    }
    pub fn audit(&self, operation: &str, detail: &str) -> Result<()> {
        self.0
            .execute(
                "INSERT INTO audit(operation,detail) VALUES (?1,?2)",
                params![operation, detail],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn record(&self, report: &serde_json::Value) -> Result<()> {
        self.0
            .execute(
                "INSERT INTO build(report) VALUES (?1)",
                [report.to_string()],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn history(&self) -> Result<Vec<serde_json::Value>> {
        let mut statement = self
            .0
            .prepare("SELECT id,created,report FROM build ORDER BY id DESC LIMIT 100")
            .map_err(db_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|row| {
            let (id, created, report) = row.map_err(db_error)?;
            let report: serde_json::Value =
                serde_json::from_str(&report).map_err(|e| invalid(e.to_string()))?;
            Ok(serde_json::json!({"id":id,"created_utc":created,"report":report}))
        })
        .collect()
    }
    pub fn workspaces(&self) -> Result<Vec<String>> {
        let mut statement = self
            .0
            .prepare("SELECT root FROM workspace ORDER BY root")
            .map_err(db_error)?;
        statement
            .query_map([], |row| row.get(0))
            .map_err(db_error)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db_error)
    }
}

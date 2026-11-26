use klab_core::{Result, invalid, sha256};
use klab_workspace::Adapter;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Instant,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::Command,
    sync::{mpsc, oneshot},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub arch: String,
    pub jobs: u16,
    pub target: String,
    #[serde(default)]
    pub llvm: bool,
    pub cross_compile: Option<String>,
    pub build_config: Option<String>,
    pub vendor_command: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Invocation {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: PathBuf,
}

fn bazel(root: &Path) -> String {
    if root.join("tools/bazel").is_file() {
        "./tools/bazel"
    } else {
        "bazel"
    }
    .into()
}

pub fn targets(root: &Path, trusted: bool) -> Result<String> {
    if !trusted {
        return Err(invalid(
            "target discovery executes the workspace Bazel launcher; review it and pass --trust",
        ));
    }
    klab_core::output(
        std::process::Command::new(bazel(root))
            .current_dir(root)
            .args([
                "query",
                "kind('kernel_build|kernel_abi|copy_to_dist_dir', //...)",
            ]),
    )
}

impl Profile {
    pub fn plan(&self, root: &Path, out: &Path, adapter: Adapter) -> Result<Invocation> {
        if !matches!(self.arch.as_str(), "arm" | "arm64" | "x86" | "x86_64") || self.jobs == 0 {
            return Err(invalid(
                "select arm, arm64, x86 or x86_64 and at least one job",
            ));
        }
        if self.target.is_empty()
            || self.target.starts_with('-')
            || self.target.contains('=')
            || self.target.chars().any(char::is_whitespace)
        {
            return Err(invalid("invalid target"));
        }
        let out = out
            .to_str()
            .ok_or_else(|| invalid("output path must be UTF-8"))?;
        let mut env = Vec::new();
        let (program, args) = match adapter {
            Adapter::Kbuild => {
                let mut args = vec![
                    format!("O={out}"),
                    format!("ARCH={}", self.arch),
                    format!("-j{}", self.jobs),
                ];
                if self.llvm {
                    args.push("LLVM=1".into());
                }
                if let Some(prefix) = &self.cross_compile {
                    args.push(format!("CROSS_COMPILE={prefix}"));
                }
                args.extend(["--".into(), self.target.clone()]);
                ("make".into(), args)
            }
            Adapter::Legacy => {
                let config = self
                    .build_config
                    .clone()
                    .ok_or_else(|| invalid("legacy builds require build_config"))?;
                env.extend([
                    ("BUILD_CONFIG".into(), config),
                    ("OUT_DIR".into(), out.into()),
                    ("DIST_DIR".into(), format!("{out}/dist")),
                    ("MAKEFLAGS".into(), format!("-j{}", self.jobs)),
                ]);
                if self.llvm {
                    env.push(("LLVM".into(), "1".into()));
                }
                if let Some(prefix) = &self.cross_compile {
                    env.push(("CROSS_COMPILE".into(), prefix.clone()));
                }
                env.push(("ARCH".into(), self.arch.clone()));
                let script = if root.join("build/build.sh").is_file() {
                    "./build/build.sh"
                } else {
                    "./build.sh"
                };
                ("bash".into(), vec![script.into(), self.target.clone()])
            }
            Adapter::Kleaf => {
                if !self.target.starts_with("//") {
                    return Err(invalid("Kleaf requires an explicit //package:target"));
                }
                (
                    bazel(root),
                    vec![
                        format!("--output_base={out}"),
                        "build".into(),
                        format!("--jobs={}", self.jobs),
                        self.target.clone(),
                    ],
                )
            }
            Adapter::Vendor => {
                let (program, args) = self
                    .vendor_command
                    .as_deref()
                    .and_then(<[String]>::split_first)
                    .ok_or_else(|| {
                        invalid("vendor_command requires an executable and optional arguments")
                    })?;
                env.extend([
                    ("OUT_DIR".into(), out.into()),
                    ("ARCH".into(), self.arch.clone()),
                    ("JOBS".into(), self.jobs.to_string()),
                ]);
                (program.clone(), args.to_vec())
            }
        };
        Ok(Invocation {
            program,
            args,
            env,
            cwd: root.to_owned(),
        })
    }
}

#[derive(Serialize)]
pub struct Diagnostic {
    file: String,
    line: u32,
    column: u32,
    severity: String,
}

#[derive(Serialize)]
pub struct Event {
    pub stream: &'static str,
    pub text: String,
    pub diagnostic: Option<Diagnostic>,
}

#[derive(Serialize)]
pub struct Report {
    pub success: bool,
    pub cancelled: bool,
    pub exit_code: Option<i32>,
    pub elapsed_ms: u128,
    pub invocation: Invocation,
    pub artifacts: Vec<Artifact>,
    pub artifact_errors: Vec<String>,
}

#[derive(Serialize)]
pub struct Artifact {
    path: PathBuf,
    sha256: String,
    bytes: u64,
}

async fn stream<R: AsyncRead + Unpin>(
    reader: R,
    name: &'static str,
    tx: mpsc::Sender<Event>,
    pattern: regex::Regex,
) -> Result<()> {
    let mut reader = BufReader::new(reader);
    loop {
        let mut bytes = Vec::new();
        let mut bounded = tokio::io::AsyncReadExt::take(&mut reader, 65536);
        if bounded.read_until(b'\n', &mut bytes).await? == 0 {
            break;
        }
        let text = String::from_utf8_lossy(&bytes)
            .trim_end_matches(['\r', '\n'])
            .to_owned();
        let diagnostic = pattern.captures(&text).and_then(|c| {
            Some(Diagnostic {
                file: c[1].into(),
                line: c[2].parse().ok()?,
                column: c[3].parse().ok()?,
                severity: c[4].into(),
            })
        });
        if tx
            .send(Event {
                stream: name,
                text,
                diagnostic,
            })
            .await
            .is_err()
        {
            break;
        }
    }
    Ok(())
}

pub async fn run(
    invocation: Invocation,
    out: &Path,
    trusted: bool,
    cancel: oneshot::Receiver<()>,
    tx: mpsc::Sender<Event>,
) -> Result<Report> {
    if !trusted {
        return Err(invalid(
            "build executes workspace code; review the invocation and pass --trust",
        ));
    }
    let start = Instant::now();
    let mut command = Command::new(&invocation.program);
    command
        .args(&invocation.args)
        .envs(invocation.env.iter().cloned())
        .current_dir(&invocation.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let pattern =
        regex::Regex::new(r"^(.+?):([0-9]+):([0-9]+): (fatal error|error|warning|note):").unwrap();
    let mut child = command.spawn()?;
    #[cfg(unix)]
    let pid = child.id().unwrap() as i32;
    let stdout = tokio::spawn(stream(
        child.stdout.take().unwrap(),
        "stdout",
        tx.clone(),
        pattern.clone(),
    ));
    let stderr = tokio::spawn(stream(child.stderr.take().unwrap(), "stderr", tx, pattern));
    let mut cancelled = false;
    let status = tokio::select! {
        status = child.wait() => status?,
        signal = cancel => {
            if signal.is_ok() {
                cancelled = true;
                #[cfg(unix)]
                {
                    use nix::{sys::signal::{kill, Signal}, unistd::Pid};
                    match kill(Pid::from_raw(-pid), Signal::SIGKILL) {
                        Ok(()) | Err(nix::errno::Errno::ESRCH) => {}
                        Err(error) => return Err(invalid(error.to_string())),
                    }
                }
                #[cfg(not(unix))]
                child.start_kill()?;
            }
            child.wait().await?
        }
    };
    for task in [stdout, stderr] {
        tokio::time::timeout(std::time::Duration::from_secs(5), task)
            .await
            .map_err(|_| invalid("build log pipe remained open"))?
            .map_err(|e| invalid(e.to_string()))??;
    }
    let elapsed_ms = start.elapsed().as_millis();
    let mut artifacts = Vec::new();
    let mut artifact_errors = Vec::new();
    if status.success()
        && !cancelled
        && let Err(error) = discover(out, 12, &mut artifacts)
    {
        artifact_errors.push(error.to_string());
    }
    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Report {
        success: status.success() && !cancelled,
        cancelled,
        exit_code: status.code(),
        elapsed_ms,
        invocation,
        artifacts,
        artifact_errors,
    })
}

fn discover(dir: &Path, depth: u8, artifacts: &mut Vec<Artifact>) -> Result<()> {
    if depth == 0 || !dir.is_dir() {
        return Ok(());
    }
    for item in dir.read_dir()? {
        let item = item?;
        let file_type = item.file_type()?;
        let path = item.path();
        if file_type.is_dir() {
            discover(&path, depth - 1, artifacts)?;
        } else if file_type.is_file()
            && (matches!(
                item.file_name().to_str(),
                Some(
                    "vmlinux"
                        | "Image"
                        | "Image.gz"
                        | "bzImage"
                        | "System.map"
                        | "Module.symvers"
                        | "zImage"
                )
            ) || matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("ko" | "dtb" | "dtbo")
            ))
        {
            artifacts.push(Artifact {
                bytes: item.metadata()?.len(),
                sha256: sha256(&path)?,
                path,
            });
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_terminates_process_group() {
        let stage = tempfile::tempdir().unwrap();
        let invocation = Invocation {
            program: "sh".into(),
            args: vec!["-c".into(), "printf 'started\\n'; sleep 30 & wait".into()],
            env: vec![],
            cwd: stage.path().into(),
        };
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let (tx, mut rx) = mpsc::channel::<Event>(16);
        let confirm = async move {
            assert_eq!(rx.recv().await.unwrap().text, "started");
            cancel_tx.send(()).unwrap();
        };
        let (report, _) = tokio::time::timeout(std::time::Duration::from_secs(4), async {
            tokio::join!(run(invocation, stage.path(), true, cancel_rx, tx), confirm)
        })
        .await
        .unwrap();
        let report = report.unwrap();
        assert!(report.cancelled);
        assert!(!report.success);
    }
}

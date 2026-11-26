#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use klab_core::{Result, invalid, read, read_text};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    kind: Kind,
    path: PathBuf,
    secondary: PathBuf,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Overview,
    Source,
    Build,
    Config,
    Dtb,
    Elf,
    Abi,
    Module,
    Boot,
    Diagnostics,
    Emulator,
    Git,
}

fn source(root: &Path, relative: &Path) -> Result<String> {
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(invalid("Expected a relative path inside the workspace"));
    }
    let root = root.canonicalize()?;
    let file = root.join(relative).canonicalize()?;
    if !file.starts_with(&root) || !file.is_file() {
        return Err(invalid(
            "Expected a regular source file inside the workspace",
        ));
    }
    String::from_utf8(read(&file, 2 * 1024 * 1024)?).map_err(|error| invalid(error.to_string()))
}

fn dispatch(request: Request) -> Result<Value> {
    let Request {
        kind,
        path,
        secondary,
    } = request;
    Ok(match kind {
        Kind::Overview => json!(klab_workspace::inspect(&path)?),
        Kind::Source => json!(source(&path, &secondary)?),
        Kind::Build => json!(klab_workspace::Store::open(&path.join(".klab.sqlite3"))?.history()?),
        Kind::Config => json!(klab_kconfig::parse(&read_text(&path)?)?),
        Kind::Dtb => json!(klab_devicetree::parse(&read(&path, 16 * 1024 * 1024)?)?),
        Kind::Elf => klab_elf::inspect(&path)?,
        Kind::Abi => klab_abi::compare(&path, &secondary)?,
        Kind::Module => json!(klab_modules::inspect(&path)?),
        Kind::Boot => json!(klab_bootimg::parse(&read(&path, 512 * 1024 * 1024)?)?),
        Kind::Diagnostics => json!(klab_diagnostics::analyze(&read_text(&path)?)),
        Kind::Emulator => json!(klab_emulator::available()),
        Kind::Git => json!(klab_git::status(&path)?),
    })
}

#[tauri::command]
async fn inspect(request: Request) -> std::result::Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || dispatch(request))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BuildRequest {
    root: PathBuf,
    profile: PathBuf,
    out: PathBuf,
}

#[derive(Default)]
struct BuildState(std::sync::Mutex<Option<Option<tokio::sync::oneshot::Sender<()>>>>);

fn build_plan(request: &BuildRequest) -> Result<(PathBuf, PathBuf, klab_build::Invocation)> {
    let root = request.root.canonicalize()?;
    let profile: klab_build::Profile = serde_json::from_str(&read_text(&request.profile)?)
        .map_err(|error| invalid(error.to_string()))?;
    let adapter = if profile.vendor_command.is_some() {
        klab_workspace::Adapter::Vendor
    } else {
        klab_workspace::inspect(&root)?.adapter
    };
    let out = std::path::absolute(&request.out)?;
    let invocation = profile.plan(&root, &out, adapter)?;
    Ok((root, out, invocation))
}

#[tauri::command]
async fn preview_build(request: BuildRequest) -> std::result::Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        build_plan(&request).map(|(_, _, plan)| json!(plan))
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn run_build(
    request: BuildRequest,
    reviewed: Value,
    events: tauri::ipc::Channel<klab_build::Event>,
    state: tauri::State<'_, BuildState>,
) -> std::result::Result<Value, String> {
    let (cancel, receiver) = tokio::sync::oneshot::channel();
    {
        let mut active = state.0.lock().map_err(|error| error.to_string())?;
        if active.is_some() {
            return Err("A build is already running.".into());
        }
        *active = Some(Some(cancel));
    }
    let result = async {
        let (root, out, invocation) =
            tauri::async_runtime::spawn_blocking(move || build_plan(&request))
                .await
                .map_err(|error| invalid(error.to_string()))??;
        if json!(invocation) != reviewed {
            return Err(invalid(
                "The build plan changed. Preview it again before running.",
            ));
        }
        let db = root.join(".klab.sqlite3");
        let store = klab_workspace::Store::open(&db)?;
        store.audit("desktop_build", &reviewed.to_string())?;
        drop(store);
        let (tx, mut rx) = tokio::sync::mpsc::channel(128);
        let task = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if events.send(event).is_err() {
                    break;
                }
            }
        });
        let report = klab_build::run(invocation, &out, true, receiver, tx).await;
        let _ = task.await;
        let report = json!({"build": report?, "git_head": klab_git::commit(&root).ok()});
        klab_workspace::Store::open(&db)?.record(&report)?;
        Ok::<_, klab_core::Error>(report)
    }
    .await;
    *state.0.lock().map_err(|error| error.to_string())? = None;
    result.map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_build(state: tauri::State<'_, BuildState>) -> std::result::Result<(), String> {
    // Keep the active slot reserved until the running build has finished.
    let mut active = state.0.lock().map_err(|error| error.to_string())?;
    if let Some(sender) = active.as_mut().and_then(Option::take) {
        let _ = sender.send(());
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(BuildState::default())
        .invoke_handler(tauri::generate_handler![
            inspect,
            preview_build,
            run_build,
            cancel_build
        ])
        .run(tauri::generate_context!())
        .expect("failed to start KERNEL//LAB");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_returns_workspace_and_rejects_source_escape() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/kbuild");
        let workspace = dispatch(Request {
            kind: Kind::Overview,
            path: root.clone(),
            secondary: PathBuf::new(),
        })
        .unwrap();
        assert_eq!(workspace["adapter"], "kbuild");
        let preview = dispatch(Request {
            kind: Kind::Source,
            path: root.clone(),
            secondary: PathBuf::from("fixture.c"),
        })
        .unwrap();
        assert!(preview.as_str().unwrap().contains("int main(void)"));
        assert!(source(&root, Path::new("../synthetic/panic.log")).is_err());
    }
}

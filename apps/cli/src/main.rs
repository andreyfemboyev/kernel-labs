use clap::{CommandFactory, Parser, Subcommand};
use klab_core::{Result, invalid, read, read_text};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(version, about = "KERNEL//LAB — Build beneath Android.")]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true, default_value = ".klab.sqlite3")]
    database: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Doctor,
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    Toolchain {
        #[command(subcommand)]
        action: ToolchainAction,
    },
    Build {
        #[command(subcommand)]
        action: Option<BuildAction>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value = "out")]
        out: PathBuf,
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long)]
        trust: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Dtb {
        #[command(subcommand)]
        action: DtbAction,
    },
    Elf {
        #[command(subcommand)]
        action: ElfAction,
    },
    Module {
        #[command(subcommand)]
        action: ModuleAction,
    },
    Abi {
        #[command(subcommand)]
        action: AbiAction,
    },
    Boot {
        #[command(subcommand)]
        action: BootAction,
    },
    Panic {
        #[command(subcommand)]
        action: PanicAction,
    },
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },
    Emulator {
        #[command(subcommand)]
        action: EmulatorAction,
    },
    Git {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[command(subcommand)]
        action: GitAction,
    },
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },
    Completions {
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum ReportAction {
    Export { input: PathBuf },
}
#[derive(Subcommand)]
enum WorkspaceAction {
    Init,
    Import {
        path: PathBuf,
    },
    Inspect {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    List,
}
#[derive(Subcommand)]
enum ToolchainAction {
    Detect,
}
#[derive(Subcommand)]
enum BuildAction {
    History,
    Targets,
}
#[derive(Subcommand)]
enum ConfigAction {
    Inspect {
        file: PathBuf,
    },
    Diff {
        baseline: PathBuf,
        candidate: PathBuf,
    },
    Validate {
        file: PathBuf,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value = "arm64")]
        arch: String,
        #[arg(long)]
        trust: bool,
    },
}
#[derive(Subcommand)]
enum DtbAction {
    Inspect {
        file: PathBuf,
    },
    Compile {
        file: PathBuf,
        #[arg(short = 'I', long)]
        include: Vec<PathBuf>,
    },
    Decompile {
        file: PathBuf,
    },
}
#[derive(Subcommand)]
enum ElfAction {
    Inspect {
        file: PathBuf,
    },
    Symbols {
        file: PathBuf,
        #[arg(long, default_value = "")]
        search: String,
    },
    Symbolize {
        file: PathBuf,
        address: String,
    },
    Disassemble {
        file: PathBuf,
    },
}
#[derive(Subcommand)]
enum ModuleAction {
    Inspect {
        file: PathBuf,
    },
    Compare {
        file: PathBuf,
        symvers: PathBuf,
        #[arg(long)]
        release: Option<String>,
    },
}
#[derive(Subcommand)]
enum AbiAction {
    Compare {
        baseline: PathBuf,
        candidate: PathBuf,
    },
}
#[derive(Subcommand)]
enum BootAction {
    Inspect {
        file: PathBuf,
    },
    Extract {
        file: PathBuf,
        component: String,
        destination: PathBuf,
    },
}
#[derive(Subcommand)]
enum PanicAction {
    Analyze {
        file: PathBuf,
        #[arg(long, requires = "address")]
        vmlinux: Option<PathBuf>,
        #[arg(long, requires = "vmlinux")]
        address: Vec<String>,
    },
}
#[derive(Subcommand)]
enum DeviceAction {
    List {
        #[arg(long)]
        fastboot: bool,
    },
    Kernel {
        serial: String,
    },
    Logs {
        serial: String,
    },
}
#[derive(Subcommand)]
enum EmulatorAction {
    List,
    QemuArm64 {
        kernel: PathBuf,
        initrd: PathBuf,
        #[arg(long)]
        approve: bool,
    },
}
#[derive(Subcommand)]
enum GitAction {
    Status,
    Diff,
    History,
    Patch {
        file: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        approve: bool,
    },
    Clone {
        url: String,
        destination: PathBuf,
        #[arg(long)]
        approve: bool,
    },
}
#[derive(Subcommand)]
enum PluginAction {
    Inspect {
        manifest: PathBuf,
    },
    Export {
        manifest: PathBuf,
        report: PathBuf,
        #[arg(long)]
        approve: bool,
    },
}

fn address(value: &str) -> Result<u64> {
    u64::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16)
        .map_err(|e| invalid(e.to_string()))
}

async fn execute(cli: Cli) -> Result<(Option<Value>, bool)> {
    let store = || klab_workspace::Store::open(&cli.database);
    let v = match cli.command {
        Command::Doctor
        | Command::Toolchain {
            action: ToolchainAction::Detect,
        } => json!(klab_toolchain::detect()),
        Command::Workspace { action } => match action {
            WorkspaceAction::Init => {
                store()?;
                json!({"database":cli.database})
            }
            WorkspaceAction::Import { path } => json!(store()?.import(&path)?),
            WorkspaceAction::Inspect { path } => {
                let ws = klab_workspace::inspect(&path)?;
                json!({"workspace":ws,"git_head":klab_git::commit(&ws.root).ok()})
            }
            WorkspaceAction::List => json!(store()?.workspaces()?),
        },
        Command::Build {
            action: Some(BuildAction::History),
            ..
        } => json!(store()?.history()?),
        Command::Build {
            action: Some(BuildAction::Targets),
            root,
            trust,
            ..
        } => json!(klab_build::targets(&root.canonicalize()?, trust)?),
        Command::Build {
            action: None,
            root,
            out,
            profile,
            trust,
            dry_run,
        } => {
            let file = profile.ok_or_else(|| invalid("build requires --profile <file.json>"))?;
            let profile: klab_build::Profile =
                serde_json::from_str(&read_text(&file)?).map_err(|e| invalid(e.to_string()))?;
            let root = root.canonicalize()?;
            let adapter = if profile.vendor_command.is_some() {
                klab_workspace::Adapter::Vendor
            } else {
                klab_workspace::inspect(&root)?.adapter
            };
            let out = std::path::absolute(out)?;
            let invocation = profile.plan(&root, &out, adapter)?;
            if dry_run {
                return Ok((Some(json!(invocation)), true));
            }
            if !trust {
                return Err(invalid(
                    "run --dry-run to review the invocation, then use --trust to execute it",
                ));
            }
            let store = store()?;
            store.audit("build", &json!(invocation).to_string())?;
            tracing::info!(?adapter, "starting build");
            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
            let signal = tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    let _ = cancel_tx.send(());
                }
            });
            let (tx, mut rx) = tokio::sync::mpsc::channel(128);
            let display = tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    let event: klab_build::Event = event;
                    if cli.json {
                        eprintln!("{}", json!(event));
                    } else {
                        eprintln!("[{}] {}", event.stream, event.text);
                    }
                }
            });
            let report = klab_build::run(invocation, &out, trust, cancel_rx, tx).await;
            signal.abort();
            display.await.map_err(|e| invalid(e.to_string()))?;
            let report = report?;
            let data =
                json!({"build":report,"profile":profile,"git_head":klab_git::commit(&root).ok()});
            store.record(&data)?;
            return Ok((Some(data), report.success));
        }
        Command::Config { action } => match action {
            ConfigAction::Inspect { file } => json!(config(&file)?),
            ConfigAction::Diff {
                baseline,
                candidate,
            } => json!(klab_kconfig::diff(
                &config(&baseline)?,
                &config(&candidate)?
            )),
            ConfigAction::Validate {
                file,
                root,
                arch,
                trust,
            } => {
                let before = config(&file)?;
                let resolved = klab_kconfig::resolve(&root, &file, &arch, trust)?;
                json!({"changes":klab_kconfig::diff(&before,&resolved),"effective":resolved})
            }
        },
        Command::Dtb { action } => match action {
            DtbAction::Inspect { file } => {
                json!(klab_devicetree::parse(&read(&file, 16 * 1024 * 1024)?)?)
            }
            DtbAction::Compile { file, include } => {
                klab_devicetree::convert(&file, false, &include)?;
                json!(file.canonicalize()?.with_extension("klab.dtb"))
            }
            DtbAction::Decompile { file } => {
                klab_devicetree::convert(&file, true, &[])?;
                json!(file.canonicalize()?.with_extension("klab.dts"))
            }
        },
        Command::Elf { action } => match action {
            ElfAction::Inspect { file } => klab_elf::inspect(&file)?,
            ElfAction::Symbols { file, search } => {
                let bytes = klab_elf::bytes(&file)?;
                json!(
                    klab_elf::symbols(&klab_elf::parse(&bytes)?)
                        .iter()
                        .filter(|s| s.name.contains(&search))
                        .collect::<Vec<_>>()
                )
            }
            ElfAction::Symbolize { file, address: hex } => {
                json!(klab_elf::symbolize(&file, address(&hex)?)?)
            }
            ElfAction::Disassemble { file } => json!(klab_elf::disassemble(&file)?),
        },
        Command::Module {
            action: ModuleAction::Inspect { file },
        } => json!(klab_modules::inspect(&file)?),
        Command::Module {
            action:
                ModuleAction::Compare {
                    file,
                    symvers,
                    release,
                },
        } => klab_modules::compatibility(&file, &symvers, release.as_deref())?,
        Command::Abi {
            action:
                AbiAction::Compare {
                    baseline,
                    candidate,
                },
        } => klab_abi::compare(&baseline, &candidate)?,
        Command::Boot { action } => match action {
            BootAction::Inspect { file } => {
                json!({"image":klab_bootimg::parse(&read(&file,512*1024*1024)?)?,"sha256":klab_core::sha256(&file)?})
            }
            BootAction::Extract {
                file,
                component,
                destination,
            } => {
                klab_bootimg::extract(&read(&file, 512 * 1024 * 1024)?, &component, &destination)?;
                json!(destination)
            }
        },
        Command::Panic {
            action:
                PanicAction::Analyze {
                    file,
                    vmlinux,
                    address: addresses,
                },
        } => {
            let log = read_text(&file)?;
            let mut symbols = Vec::new();
            if let Some(vmlinux) = vmlinux {
                for hex in addresses {
                    symbols.push(json!({"address":hex,"result":klab_elf::symbolize(&vmlinux,address(&hex)?)?}));
                }
            }
            json!({"findings":klab_diagnostics::analyze(&log),"symbolization":symbols})
        }
        Command::Device { action } => match action {
            DeviceAction::List { fastboot } => json!(klab_device::list(fastboot)?),
            DeviceAction::Kernel { serial } => json!(klab_device::kernel(&serial)?),
            DeviceAction::Logs { serial } => json!(klab_device::log(&serial)?),
        },
        Command::Emulator { action } => match action {
            EmulatorAction::List => json!(klab_emulator::available()),
            EmulatorAction::QemuArm64 {
                kernel,
                initrd,
                approve,
            } => {
                if cli.json {
                    return Err(invalid(
                        "QEMU attaches an interactive serial console; omit --json",
                    ));
                }
                if !approve {
                    return Err(invalid("emulator launch requires --approve"));
                }
                store()?.audit("emulator_launch", &json!([kernel, initrd]).to_string())?;
                let status = klab_emulator::qemu_arm64(&kernel, &initrd, approve)?.wait()?;
                return Ok((None, status.success()));
            }
        },
        Command::Git { root, action } => match action {
            GitAction::Status => json!(klab_git::status(&root)?),
            GitAction::Diff => json!(klab_git::diff(&root)?),
            GitAction::History => json!(klab_git::history(&root)?),
            GitAction::Patch {
                file,
                apply,
                approve,
            } => {
                if apply && approve {
                    store()?.audit("git_apply", &json!([root, file]).to_string())?;
                }
                json!(klab_git::patch(&root, &file, apply, approve)?)
            }
            GitAction::Clone {
                url,
                destination,
                approve,
            } => {
                if approve {
                    store()?.audit("git_clone", &destination.display().to_string())?;
                }
                json!(klab_git::clone(&url, &destination, approve)?)
            }
        },
        Command::Plugin { action } => match action {
            PluginAction::Inspect { manifest } => json!(klab_sdk::inspect(&manifest)?),
            PluginAction::Export {
                manifest,
                report,
                approve,
            } => {
                if approve {
                    store()?.audit("plugin_export", &json!([manifest, report]).to_string())?;
                }
                json!(klab_sdk::export(&manifest, &report, approve)?)
            }
        },
        Command::Report {
            action: ReportAction::Export { input },
        } => {
            let report: Value =
                serde_json::from_str(&read_text(&input)?).map_err(|e| invalid(e.to_string()))?;
            json!(format!(
                "# KERNEL//LAB report\n\n````json\n{}\n````",
                serde_json::to_string_pretty(&report).map_err(|e| invalid(e.to_string()))?
            ))
        }
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "klab", &mut std::io::stdout());
            return Ok((None, true));
        }
    };
    Ok((Some(v), true))
}

fn config(path: &Path) -> Result<klab_kconfig::Config> {
    klab_kconfig::parse(&read_text(path)?)
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::WARN)
        .init();
    match execute(cli).await {
        Ok((value, success)) => {
            if let Some(value) = value {
                if !json && let Some(text) = value.as_str() {
                    println!("{text}");
                } else {
                    println!("{}", serde_json::to_string_pretty(&value).unwrap());
                }
            }
            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            if json {
                eprintln!("{}", serde_json::json!({"error":error.to_string()}));
            } else {
                eprintln!("klab: {error}");
            }
            ExitCode::from(1)
        }
    }
}

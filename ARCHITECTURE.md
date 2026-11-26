# Architecture

The root Cargo workspace contains the CLI and reusable engine crates. The Tauri binary has an independent lockfile so a headless build can exercise the engine without a system WebView. It invokes the same workspace, artifact and diagnostic crates through typed IPC. Build requests show the planned invocation for approval, compare it again before launch, and stream output through a Tauri IPC channel. React owns panel state and renders reports and source buffers with a local Monaco bundle.

`klab-core` supplies bounded file reads, checked binary reads, checksums and shared error types. `klab-workspace` detects source tree shape and persists imports, build reports and audit records in SQLite. A build profile becomes an executable, argument vector, environment pairs and working directory. Tokio streams stdout and stderr through a bounded channel and handles cancellation. The CLI writes the final structured report on stdout and build events on stderr.

Artifact crates inspect local inputs and explicitly invoke optional native tools where appropriate. The plugin SDK uses a versioned manifest and an executable invoked as a subprocess.

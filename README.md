# KERNEL//LAB

**Build beneath Android.** An offline-first engineering workbench for Android and Linux kernel source trees and artifacts.

[![CI](https://github.com/andreyfemboyev/kernel-labs/actions/workflows/ci.yml/badge.svg)](https://github.com/andreyfemboyev/kernel-labs/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Releases](https://github.com/andreyfemboyev/kernel-labs/releases) · [Architecture](ARCHITECTURE.md) · [Verification](docs/VERIFICATION.md) · [Portfolio](https://andreyfemboyev.github.io/portfolio/#kernel-labs)

## What you can do

- Inspect local kernel trees, build profiles, toolchains and SQLite build history.
- Preview and approve build invocations; stream logs and record artifact checksums.
- Inspect ELF, DTB, Kconfig, kernel modules, ABI metadata and boot images.
- Analyze panic logs with heuristics and optional external tools.
- Use a Tauri desktop workbench or the standalone Rust CLI.

**Alpha scope:** the native synthetic Kbuild fixture is verified; full Android builds, live-device operations and additional desktop targets need integration testing. No flashing.

## Quick start

`v0.1.0-alpha` includes a standalone Rust CLI and a Tauri 2 desktop source tree. Start with [Building](BUILDING.md), the [compatibility matrix](docs/LIMITATIONS.md), and the [build adapter guide](docs/BUILD_ADAPTERS.md).

```sh
cargo build --locked --release -p klab
klab=./target/release/klab
root=./fixtures/kbuild
"$klab" doctor
"$klab" workspace import "$root"
"$klab" build --root "$root" --profile "$root/profile.json" --dry-run
"$klab" build --root "$root" --profile "$root/profile.json" --trust
"$klab" elf inspect ./out/vmlinux
"$klab" --json panic analyze ./fixtures/synthetic/panic.log
```

The fixture uses Make and a native C compiler to produce a synthetic ELF, so it can test the build pipeline without downloading an Android kernel. For a real tree, use its branch-specific tooling and configuration. [Android kernel build guide](https://source.android.com/docs/setup/build/building-kernels) · [Android kernel architecture](https://source.android.com/docs/core/architecture/kernel)

Original code is MIT licensed. Consult [third-party license information](docs/THIRD_PARTY_LICENSES.md) when redistributing.

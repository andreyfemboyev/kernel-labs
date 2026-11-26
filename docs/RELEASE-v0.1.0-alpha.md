# KERNEL//LAB v0.1.0-alpha

First public alpha candidate — an offline-first Android/Linux kernel engineering workbench.

## Highlights
- Rust CLI (klab) and Tauri 2 / React desktop workbench.
- Workspace discovery and SQLite history; Kbuild, Kleaf, legacy and vendor build plans.
- Explicit build review and trust gate, streamed logs and artifact SHA-256.
- Kconfig, DTB, ELF, modules, Module.symvers, boot images and heuristic panic inspection.
- Optional ADB/Fastboot discovery, QEMU ARM64 invocation and export plugins.

## Scope
The synthetic Kbuild fixture compiles a real ELF with the native C toolchain; this is not a full Android kernel build. Linux desktop smoke tests are documented. Hosted CI run [45813087483](https://github.com/andreyfemboyev/kernel-labs/actions/runs/45813087483) passed Rust checks on Linux x86-64, ARM64 and Windows and frontend checks. Packaging and live-device validation are separate. No flashing or AVB/signature verification. Do not execute untrusted build profiles.

## Downloads
Release candidates contain platform-specific CLI and desktop bundles, source archive, license and SHA256SUMS. Verify checksums before unpacking or installing; see BUILDING.md and docs/LIMITATIONS.md. Windows/ARM64 desktop execution is not implied by CI success.

MIT licensed original code; see docs/THIRD_PARTY_LICENSES.md.

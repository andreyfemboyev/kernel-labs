# Compatibility matrix

| Area | Available in this alpha | Verification and limits |
| --- | --- | --- |
| Workspace | Local import, detection, SQLite list, HTTPS Git clone, HEAD inspection | Local fixture exercised. Repo manifest sync, snapshots and source indexing pending. |
| Build | Kbuild, Kleaf, legacy and vendor invocation, logs, history, cancellation and checksums | Native Make fixture and Unix process-group cancellation tested. Full Android, alternate adapters and cross-toolchains require integration verification. |
| Kconfig | Configuration value inspection, diff and kernel olddefconfig preview | Parser unit tested. Menus, reverse dependencies, fragment workflow and graphical validation pending. |
| Device Tree | DTB v17 node/property inspection and optional dtc conversion | Synthetic DTB tested. Overlay application, DTBO containers, schema validation and navigation pending. |
| Binary | ELF headers, sections, symbols, BTF presence and optional LLVM tools | Real fixture ELF inspected. Raw Image, Image.gz, maps, BTF decoding and binary comparison pending. |
| ABI and modules | Module.symvers comparison, ELF module metadata, symbol lists, release and import checks, and signature marker | Synthetic CRC comparison tested. STG, module CRC comparison, signature verification and module build/package workflow pending. |
| Boot images | Boot v0–v4, vendor boot v3/v4 and component extraction, including init_boot layouts that use v4 | Synthetic boot v3 tested. Unsupported versions return errors. No ramdisk filesystem extraction, AVB verification or packing. |
| Diagnostics | Text heuristics and explicit address symbolization with LLVM | Classification is heuristic. Automatic frame extraction, device session comparison and source linking pending. |
| Devices and emulation | ADB discovery and diagnostics, Fastboot discovery, QEMU ARM64 invocation | External device and emulator operations require integration verification. No flashing. |
| Git and SDK | Status, diff, log, patch check/apply, clone and protocol-1 export plugins | Patch series, branch operations and additional plugin types pending. |
| Desktop | Tauri IPC, build plan approval and streaming logs, artifact reports, Monaco scratch buffer, command palette, themes, English/Russian UI labels | Native Linux workspace and build paths were exercised through WebKitGTK. Editor persistence, integrated terminal, and specialized studio panels remain pending. |
| Packaging | Linux x86_64 CLI, Debian desktop package, source distribution, and packaging workflows | Windows and ARM64 still require execution on a suitable host. |

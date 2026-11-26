# KERNEL//LAB

**Build beneath Android.** An offline-first engineering workbench for Android and Linux kernel source trees and artifacts.

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

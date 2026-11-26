# Building and installing

Use stable Rust with rustfmt and Clippy, Git, and a native C toolchain for SQLite. On Linux the fixture also requires Make. Build the CLI with `cargo build --locked --release -p klab`, then install `target/release/klab` in a directory on your PATH. The Linux x86_64 CLI archive carries the executable and this guide. It was built on Debian 13. Compare its SHA-256 against the distribution manifest before unpacking.

Run the CLI inside your Linux distro on Windows with WSL2 and keep working trees within the distro filesystem. Direct Windows CLI use covers offline inspection and depends on separately installed executables for optional tooling. The desktop app does not bridge build requests to WSL2.

## Desktop

The Linux x86_64 Debian build requires GTK 3 and WebKitGTK 4.1. Run `apt install ./kernel-lab-desktop-linux-x86_64.deb` on Debian 13 with administrator privileges, then launch KERNEL LAB from the application menu. The local package uses the debug Rust profile with the production frontend embedded.


On Debian or Ubuntu install the Tauri prerequisites, including `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `pkg-config`, and a C compiler. On Windows use the MSVC Rust toolchain, Visual Studio C++ Build Tools and WebView2. Install Node.js 22.13 or newer on a supported LTS release.

```sh
cd apps/desktop
npm ci
npm run tauri -- dev
npm run tauri -- build
```

Use `npm run tauri -- build --bundles deb,appimage` on a supported Linux packaging host and `npm run tauri -- build --bundles nsis` on Windows. Bundles appear under `src-tauri/target/release/bundle`. Build dependencies require an initial download or prepopulated caches. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/desktop
npm run typecheck
npm run lint
npm test
npm run build
```

An optional real integration run is available via `scripts/android-smoke.sh`. Review your source tree and build plan first. Its arguments are the built klab path, source root, JSON profile and a new output directory.

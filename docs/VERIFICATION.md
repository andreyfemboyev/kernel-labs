# v0.1.0-alpha verification

Checks ran on Debian 13 x86_64 with Rust 1.100.0 and Node.js 24.22.3.

| Check | Result |
| --- | --- |
| Cargo format and workspace Clippy with warnings denied | Passed |
| Rust workspace tests | 10 passed |
| Tauri backend dispatch test | 1 passed |
| Frontend type check and lint | Passed |
| Frontend IPC request mapping test | 1 passed |
| Production frontend bundle | Passed. Vite reported a Monaco bundle size warning. |
| npm dependency audit | Zero reported vulnerabilities after updating the test runner. |
| CLI release build and archive smoke test | Passed. Extracted CLI returned `klab 0.1.0-alpha`. |
| Native desktop workspace inspection | Passed. The WebView displayed the Kbuild fixture version and architecture from real IPC. |
| Native desktop build flow | Passed. The UI previewed and ran Make, the fixture produced an ELF, and SQLite history recorded a successful build with SHA-256. |
| Debian package | Built and inspected with dpkg-deb. The extracted executable opened a native window. |
| AppImage | Built and exercised in extract-and-run mode. The virtual display used software-compatible WebKit settings and a host GLES library. |

The Kbuild-style fixture is a synthetic source tree compiled by a real compiler. A full Android kernel build, ARM64 and Windows packaging, and live device operations were outside the verified test matrix. Hosted [CI run 45813087483](https://github.com/andreyfemboyev/kernel-labs/actions/runs/45813087483) subsequently passed all four jobs (Linux x86-64, Linux ARM64, Windows Rust, and frontend). This is not a desktop packaging or device smoke test.

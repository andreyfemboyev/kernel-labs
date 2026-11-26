# Security

Imported trees, JSON profiles, tools on PATH, repository configuration, and plugins may execute native code when invoked. Review them before supplying `--trust` or `--approve`. Approval applies to the requested operation. Process groups and subprocess boundaries provide cancellation and interface isolation. They provide no filesystem or network sandbox.

Inspect untrusted material from a restricted account or disposable VM. Artifact reads are bounded, and component extraction uses an explicit destination and refuses to overwrite an existing file. Ramdisk archive unpacking is outside the implemented scope. ADB commands use an explicit device authorization context and respect the permissions granted on that device.

The CLI records build, clone, patch application, emulator launch and plugin execution requests locally in SQLite. Reports may contain paths and tool arguments. The app implements no telemetry.

Report vulnerabilities privately through your distribution maintainer or a repository private advisory once enabled. Include version, affected command, and sanitized reproduction details.

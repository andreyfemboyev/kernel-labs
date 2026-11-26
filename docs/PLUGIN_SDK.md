# Plugin SDK

Protocol 1 manifests contain `protocol`, `name`, `executable` and `capabilities`. The executable is a basename resolved beside the manifest and the supported capability list is `["export"]`. The CLI calls it as `EXECUTABLE export-v1 ABSOLUTE_REPORT_PATH` with an empty inherited environment. Output is UTF-8 on stdout, zero indicates success, and a nonzero status is reported with stderr. Plugin code executes with the user's OS permissions.

Run the sample with `klab plugin export examples/plugin/manifest.json report.json --approve` on Linux.

# Build adapters

Use `klab build --root TREE --out OUTPUT --profile PROFILE.json --dry-run` to inspect the executable, arguments, and working directory. Add `--trust` to run the reviewed invocation. Resolve output paths relative to the CLI's current directory. Use a dedicated output directory per profile. Successful reports list recognized files currently present in that output tree, which can include incremental artifacts from earlier runs.

| Adapter | Selection | Configuration |
| --- | --- | --- |
| Kbuild | Makefile and Kconfig | Set `arch`, `target`, `jobs`, optional `llvm` and `cross_compile`. Use target `clean` only when you intend to clean the selected output. |
| Kleaf | Local launcher or Kleaf tooling marker | Use `klab build --root TREE --trust targets` and choose a concrete `//package:target` from the query. The branch target defines toolchain and architecture. |
| Legacy | build/build.sh or build.sh | Set `build_config` to the branch's build configuration. Build and output conventions depend on the branch. |
| Vendor | Explicit vendor_command | Provide an executable followed by separate arguments. The adapter supplies OUT_DIR, ARCH and JOBS. |

Build scripts and toolchains can run subprocesses and access the host. Bazel may download dependencies according to its configuration. [Android kernel build guide](https://source.android.com/docs/setup/build/building-kernels)

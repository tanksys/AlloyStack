# LibOS Build and Run Principles

## Purpose and scope

This document explains the principles behind how AlloyStack's LibOS components are built and how workflows execute. It focuses on the conceptual build and execution pipeline rather than step-by-step commands.

## Key components and their roles

- **`libasvisor`**: the runtime that reads isolation configs, loads shared libraries, initializes LibOS services, and drives workflow execution.
- **`as_std`**: the standard library layer used by services and user functions for consistent APIs.
- **`common_service/`**: the LibOS service modules (e.g., `fdtab`, `stdio`, `mm`, `fatfs`, `time`) built as shared libraries.
- **`user/`**: user functions (apps) compiled into shared libraries and loaded by `libasvisor`.
- **`isol_config/`**: workflow specifications that declare which services and apps to load and how apps are grouped and parameterized.

## Build pipeline principles

The build pipeline outputs shared libraries (`lib*.so`) for both LibOS services and user apps. The high-level build graph is:

1. **LibOS services** are compiled from `common_service/` into `target/{debug|release}/lib<service>.so`.
2. **User apps** are compiled from `user/` into `target/{debug|release}/lib<app>.so`.

The `just` recipes encapsulate this graph: building all LibOS modules corresponds to compiling the service crates, while building a specific user function compiles a single app crate. The result is that both services and apps are available as shared libraries under the profile-specific `target/` directory.

## Isolation config as the contract

An isolation config (for example `isol_config/simple_file.json`) is the contract between build output and runtime behavior:

- **`services`** list the LibOS modules to load (name + shared library filename).
- **`apps`** list the user functions to load (name + shared library filename).
- **`groups`** (optional) describe staged execution and parallelism, with shared args merged into per-app arguments.
- **`fs_image`** (optional) specifies a filesystem image for file-backed workloads.
- **`with_libos`** (optional) can disable LibOS services when set to `false`.

During config loading, `libasvisor` normalizes module paths into `target/{debug|release}/...` so that the runtime can resolve the build artifacts directly.

## Execution pipeline principles

At runtime, `libasvisor` follows a consistent loading and execution flow:

1. **Parse and normalize** the isolation config, expanding module paths to the active build profile.
2. **Register** all service and app modules from the config.
3. **Load services** on demand as shared libraries, initialize them, and attach them to the runtime environment.
4. **Load apps** (user functions) as shared libraries and execute them, honoring group structure and arguments.

`libasvisor` uses a shared-library loader that supports on-demand loading and, when enabled, additional isolation features (e.g., MPK or namespace loading).

## Example: `simple_file`

The `simple_file.json` configuration declares a small workflow with LibOS services (`fdtab`, `stdio`, `mm`, `fatfs`, `time`) and a single app (`simple_file`). Conceptually:

- The build step produces `libfdtab.so`, `libstdio.so`, `libmm.so`, `libfatfs.so`, `libtime.so`, and `libsimple_file.so` under `target/{debug|release}`.
- The runtime reads the config, resolves each entry to the corresponding `target/{profile}/lib*.so` path, and registers them.
- Services are loaded and initialized first so the app can rely on LibOS APIs.
- The `simple_file` app is then loaded and executed within the configured isolation environment.

This reflects the core principle: the config names the modules, the build produces their shared libraries, and the runtime ties them together at execution time.

## How `simple_file` reaches the FS service

`simple_file` uses the `as_std::fs::File` API, which is a thin LibOS-facing wrapper over hostcall-based system calls. The call chain is:

1. **User code** calls `File::create`, `File::open`, `Read::read`, `Write::write`, and `File::seek`.
2. **`as_std`** maps those operations to `libos!` hostcalls such as `open`, `read`, `write`, `lseek`, and `stat`.
3. **`fdtab` service** implements those hostcalls and maintains the file descriptor table; for file-backed descriptors it routes to the `fatfs` service.
4. **`fatfs` service** performs actual file operations on the FAT filesystem image.

In code, `simple_file` uses `as_std::fs::File` for create/open/read/write/seek. `as_std` then invokes `libos!(open/ read/ write/ lseek/ stat)` under the hood, which resolves to the `fdtab` service functions at runtime. The `fdtab` service delegates file operations to `fatfs` by calling `libos!(fatfs_*)`, and `fatfs` performs the filesystem IO.

## How the pieces are linked together

All three layers are built as shared libraries:

- **App layer**: `libsimple_file.so` depends on `as_std` and uses the `libos!` hostcall mechanism.
- **LibOS services**: `libfdtab.so` exposes the `open/read/write/lseek/stat/close` entrypoints; `libfatfs.so` exposes `fatfs_open/fatfs_read/fatfs_write/...` entrypoints.
- **Runtime**: `libasvisor` loads these shared libraries, registers their symbols, and provides the hostcall resolution function used by `libos!`.

At runtime, `libasvisor` loads the service libraries first, so the `fdtab` and `fatfs` entrypoints are registered. When the `simple_file` app invokes `libos!(open)` (via `as_std::fs::File`), the hostcall lookup resolves to the `fdtab` symbol, which then forwards to `fatfs` for real IO. This is how the app, LibOS services, and runtime connect without static linking.

## Optional isolation features

AlloyStack supports optional isolation mechanisms that affect loading and execution:

- **MPK (Memory Protection Keys)** can isolate memory regions for services and apps.
- **Namespace-based loading** can separate dynamic library namespaces.

These options change how libraries are loaded and isolated, but they do not change the core contract between configs, build artifacts, and runtime execution.

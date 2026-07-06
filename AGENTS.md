# AlloyStack 协作指南

## 项目定位

AlloyStack 是面向 Serverless workflow 的 Rust LibOS 运行时。它在单一地址空间中：

- 按需加载用户函数及系统服务，以降低冷启动开销。
- 通过引用传递 workflow 的中间数据，减少序列化与复制。
- 让 Rust 用户函数以 `#![no_std]` 方式依赖 `as_std`，并直接链接到 LibOS，而不是 libc。
- 通过定制的 `wasmtime-as-lib` 执行 WebAssembly，并由 `wasmtime_wasi_api` 将 WASI 调用接入 LibOS，从而承载 C/CPython workload。

注意：仓库实现使用 **Wasmtime**，不是 Wasmer。讨论或修改 WebAssembly 路径时，以代码中的 `wasmtime_wasi_api` 和 `wasmtime-as-lib` 为准。

## 目录职责

- `bins/asvisor/`：主要运行时命令行入口。
- `libasvisor/`：隔离实例、模块加载、workflow 调度、指标和 hostcall 接入。
- `as_hostcall/`：LibOS hostcall 及文件、内存、网络等后端。
- `as_std/`：Rust 用户函数使用的 `no_std` 接口、分配器、panic 和数据传递支持。
- `as_std_proc_macro/`：用户函数相关过程宏。
- `common_service/`：可独立构建并按需加载的 LibOS 服务。
- `wasmtime_wasi_api/`：Wasmtime/WASI 到 LibOS 的适配层。
- `user/`：用户函数及 Wasmtime/CPython workload；不属于根 Cargo workspace。
- `isol_config/`：workflow JSON 定义，是运行时装配 workflow 的事实来源。
- `libasvisor/tests/`：运行时集成测试。
- `baseline/`：对比实现和实验脚本，不属于根 Cargo workspace。
- `fs_images/`、`image_content/`：文件系统镜像及挂载内容。

根 workspace 刻意排除了 `user/`、`common_service/` 和 `baseline/`。不要仅凭根目录 `cargo test` 判断这些组件已经构建或验证。

## 工具链与构建

项目基准环境是 Ubuntu 22.04、GCC 11.4 和 `nightly-2023-12-01`。部分实验依赖 Intel MPK、sudo、TAP 设备或已挂载的 FAT 镜像。

常用命令：

```bash
just init
just asvisor
just libos <service_name>
just rust_func <function_name>
just all_libos
target/release/asvisor --files isol_config/<workflow>.json
```

`justfile` 默认以 release 模式构建。可通过变量覆盖功能开关，例如：

```bash
just enable_mpk=1 rust_func <function_name>
just enable_file_buffer=1 rust_func <function_name>
just enable_release=0 asvisor
```

相关 feature 约束：

- `mpk`/`enable_mpk` 启用内存保护键；需要兼容硬件和运行环境。
- `pkey_per_func` 建立在 MPK 之上。
- `file-based` 用文件中转代替引用传递，主要用于消融实验。
- `as_std` 的 `unwinding` 与 `panic_def` 互斥，不得同时启用。

不要擅自运行需要 sudo、修改网络设备、挂载镜像或执行完整性能评测的 recipe。此类命令包括但不限于 `cold_start_latency`、`end_to_end_latency`、`breakdown`、`p99` 和 `resource_consume`。

## Workflow 配置

workflow 在 `isol_config/*.json` 中定义：

- `services`：需加载的 LibOS 服务及其 `.so` 文件。
- `apps`：用户函数及其 `.so` 文件。
- `groups`：按顺序执行的阶段；同一 group 中的函数可并行执行。
- group 级 `args` 会作为默认参数，单个 app 的 `args` 会覆盖同名值。
- 运行时会为 group 中每个 app 注入字符串形式的 `id`。
- 可选字段包括 `fs_image` 和 `with_libos`。

新增或修改 workflow 时，应同步确认：

1. 所有 service 和 app 名称与构建产物名称一致。
2. 所需 service 已列入 `services`，并可通过 `just libos <name>` 构建。
3. app 可通过 `just rust_func <name>` 或对应的 `wasm_func` recipe 构建。
4. group 参数均为字符串，并与函数实际读取的键一致。
5. workflow 所需数据确实存在于对应文件系统镜像中。

## 修改原则

- 保持单地址空间、按需加载和引用传递这三条核心路径的语义；不要无意中引入 libc 依赖或跨边界复制。
- 修改用户函数接口时，同时检查 `as_std`、`as_hostcall`、运行时装载逻辑和相关 workflow 配置。
- 修改 WASI 实现时，检查 `wasmtime_wasi_api/src/lib.rs` 的 linker 注册及 `wasi.rs` 中对应实现。
- 新增用户函数时遵循相邻 `user/*` crate 的布局、crate type、build script 和 feature 传递方式。
- 不要把 `user/` 或 `common_service/` 草率加入根 workspace；现有排除与 `as_std` 的冲突 feature 组合有关。
- 不要提交生成物、挂载内容、实验日志或大型数据文件，除非任务明确要求。
- 仓库含来源于 RuxOS 的代码；修改相关实现时保留既有来源和许可证信息。

## 验证要求

验证范围应与修改范围匹配，优先使用最小且有针对性的命令：

```bash
cargo fmt --all -- --check
cargo test -p libasvisor
cargo build -p asvisor
just libos <changed_service>
just rust_func <changed_function>
target/release/asvisor --files isol_config/<affected_workflow>.json
```

注意：

- 根 workspace 构建可能需要获取定制 git 依赖；网络不可用时应明确报告，不能把依赖获取失败描述成代码失败。
- 集成测试和 workflow 执行可能依赖 MPK、镜像、网络权限或已构建的动态库。报告验证结果时区分代码错误与环境前置条件不足。
- 性能相关修改不能只证明“能运行”；应使用相同构建模式、feature、数据集和 workflow 配置进行前后对比。
- 未实际执行的测试不得声称通过。

## 沟通约定

- 回答使用中文，保持结构化、直接和可核验。
- 对项目描述、代码行为和实验结论都保持审慎；发现文档与实现冲突时，以代码和可复现实验为依据，并明确指出冲突。
- 不做过度夸赞。信息不足且会影响技术结论时，主动索要配置、日志、复现步骤或实验数据。

# Python workflow sources

`wasm_bench/` is the canonical source for the CPython word-count,
parallel-sort, and function-chain workloads. Both Wasmtime CPython and native
musl CPython execute these files from `/wasm_bench` in the FAT image.

Synchronize source changes before running a workflow:

```bash
./scripts/sync_python_workloads.sh
```

If `fs_images/fatfs.img` is mounted at `image_content` and the mount is not
writable by the current user, run the command with `sudo -E`.

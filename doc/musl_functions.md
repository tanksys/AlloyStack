# Running C functions with musl

AlloyStack can build C functions against a patched musl libc. The C source uses
normal libc APIs, while musl redirects operating-system-facing calls to
AlloyStack's LibOS services.

## Prerequisites

Initialize the pinned musl submodule once:

```bash
git submodule update --init third_party/musl
```

Build the patched, position-independent musl archive:

```bash
just musl
```

## Function layout

A musl C function is a regular user crate with:

```text
user/<function>/
├── Cargo.toml
├── function.c
└── src/lib.rs
```

The Rust library is a no-std shim:

```rust
#![no_std]

as_musl::entry!(alloy_c_main);
```

The C source keeps a standard entry point:

```c
int main(int argc, char **argv)
{
    return 0;
}
```

The build script renames it to `alloy_c_main` without requiring a source edit.
Workflow arguments are exposed as `--key=value`; `argv[0]` is the function
crate name.

Build a function with:

```bash
just musl_func <function>
```

The example workflow is:

```bash
just musl_example
```

It launches four concurrent instances of the same C function and exercises
stdio, malloc, and FATFS file operations.

## Initial compatibility boundary

The first implementation supports:

- `read`, `write`, `readv`, and `writev`
- `open`, `openat(AT_FDCWD)`, `close`, `lseek`, and `fstat`
- anonymous `mmap`, `munmap`, `mprotect`, and `madvise`
- musl-internal `futex`, `gettid`, and `getrandom`

Unsupported syscalls return `ENOSYS` and are logged on their first occurrence.
The initial implementation does not support pthread creation, signals,
fork/exec, musl dynamic linking, explicit `exit`/`abort`, or prebuilt ELF
binaries. C `main` must return normally.

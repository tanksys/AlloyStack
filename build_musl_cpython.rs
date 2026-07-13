use std::{env, fs, path::PathBuf, process::Command};

fn run(command: &mut Command) {
    assert!(
        command
            .status()
            .expect("failed to execute build command")
            .success(),
        "build command failed: {:?}",
        command
    );
}

fn main() {
    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = crate_dir.parent().unwrap().parent().unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let musl_dir = repo_root.join("target/musl-alloy");
    let musl_lib = musl_dir.join("lib/libmusl_alloy.a");
    let cpython_dir = repo_root.join("target/cpython-musl");
    let python_include = PathBuf::from(env::var_os("PYTHON_INCLUDE").unwrap_or_else(|| {
        repo_root.join("third_party/cpython/Include").into_os_string()
    }));
    let python_config_include = PathBuf::from(
        env::var_os("PYTHON_CONFIG_INCLUDE")
            .unwrap_or_else(|| cpython_dir.join("build-submodule").into_os_string()),
    );
    let python_lib_dir = PathBuf::from(
        env::var_os("PYTHON_LIB_DIR")
            .unwrap_or_else(|| cpython_dir.join("build-submodule").into_os_string()),
    );
    let python_lib = python_lib_dir.join("libpython3.11.a");
    let numpy_enabled = env::var_os("CARGO_FEATURE_NUMPY").is_some();
    let numpy_dir = repo_root.join("target/numpy-musl");
    let numpy_lib_dir = numpy_dir.join("lib");
    let numpy_lib = numpy_lib_dir.join("libnumpy_musl.a");
    let cxx_runtime_dir = PathBuf::from(
        env::var_os("CXX_MUSL_PREFIX")
            .unwrap_or_else(|| repo_root.join("target/cxx-musl").into_os_string()),
    );
    let cxx_runtime_lib_dir = cxx_runtime_dir.join("lib");
    let source = crate_dir.join("function.c");
    let datetime_source = repo_root.join("third_party/cpython/Modules/_datetimemodule.c");
    let math_source = repo_root.join("third_party/cpython/Modules/mathmodule.c");
    let struct_source = repo_root.join("third_party/cpython/Modules/_struct.c");
    let contextvars_source = repo_root.join("third_party/cpython/Modules/_contextvarsmodule.c");
    let binascii_source = repo_root.join("third_party/cpython/Modules/binascii.c");

    assert!(
        musl_lib.is_file(),
        "missing {}; run `just musl` first",
        musl_lib.display()
    );
    assert!(
        python_include.join("Python.h").is_file() && python_lib.is_file(),
        "missing musl CPython; run `just musl_cpython`"
    );
    if numpy_enabled {
        assert!(
            numpy_lib.is_file(),
            "missing {}; run `python3 scripts/build_numpy_musl.py --archive-only` first",
            numpy_lib.display()
        );
    }

    let cc = env::var_os("CC").unwrap_or_else(|| "cc".into());
    let gcc_include = Command::new(&cc)
        .arg("-print-file-name=include")
        .output()
        .expect("failed to query compiler include directory");
    assert!(gcc_include.status.success());
    let gcc_include = String::from_utf8(gcc_include.stdout)
        .unwrap()
        .trim()
        .to_owned();

    let object = out_dir.join("function.o");
    let mut common_compile_args = vec![
        "-std=c11".into(),
        "-O2".into(),
        "-fPIC".into(),
        "-ffreestanding".into(),
        "-fno-stack-protector".into(),
        "-fno-builtin".into(),
        "-nostdinc".into(),
        "-D_GNU_SOURCE".into(),
        "-D_POSIX_C_SOURCE=200809L".into(),
    ];
    if numpy_enabled {
        common_compile_args.push("-DALLOY_ENABLE_NUMPY".into());
    }
    common_compile_args.extend([
        "-isystem".into(),
        musl_dir.join("include").into_os_string(),
        "-I".into(),
        repo_root.join("as_musl/include").into_os_string(),
        "-isystem".into(),
        python_config_include.clone().into_os_string(),
        "-isystem".into(),
        python_include.clone().into_os_string(),
        "-isystem".into(),
        python_include.join("internal").into_os_string(),
        "-isystem".into(),
        gcc_include.into(),
    ]);

    run(Command::new(&cc)
        .args(&common_compile_args)
        .arg("-Dmain=alloy_c_main")
        .arg("-c")
        .arg(&source)
        .arg("-o")
        .arg(&object));

    let datetime_object = out_dir.join("_datetimemodule.o");
    let math_object = out_dir.join("mathmodule.o");
    let struct_object = out_dir.join("_struct.o");
    let contextvars_object = out_dir.join("_contextvarsmodule.o");
    let binascii_object = out_dir.join("binascii.o");
    if numpy_enabled {
        run(Command::new(&cc)
            .args(&common_compile_args)
            .arg("-DPy_BUILD_CORE_MODULE")
            .arg("-c")
            .arg(&datetime_source)
            .arg("-o")
            .arg(&datetime_object));
        run(Command::new(&cc)
            .args(&common_compile_args)
            .arg("-DPy_BUILD_CORE_MODULE")
            .arg("-c")
            .arg(&math_source)
            .arg("-o")
            .arg(&math_object));
        run(Command::new(&cc)
            .args(&common_compile_args)
            .arg("-DPy_BUILD_CORE_MODULE")
            .arg("-c")
            .arg(&struct_source)
            .arg("-o")
            .arg(&struct_object));
        run(Command::new(&cc)
            .args(&common_compile_args)
            .arg("-DPy_BUILD_CORE_MODULE")
            .arg("-c")
            .arg(&contextvars_source)
            .arg("-o")
            .arg(&contextvars_object));
        run(Command::new(&cc)
            .args(&common_compile_args)
            .arg("-DPy_BUILD_CORE_MODULE")
            .arg("-c")
            .arg(&binascii_source)
            .arg("-o")
            .arg(&binascii_object));
    }

    let app_archive = out_dir.join("liballoy_cpython_app.a");
    let ar = env::var_os("AR").unwrap_or_else(|| "ar".into());
    let mut ar_command = Command::new(ar);
    ar_command.arg("rcs").arg(&app_archive).arg(&object);
    if numpy_enabled {
        ar_command
            .arg(&datetime_object)
            .arg(&math_object)
            .arg(&struct_object)
            .arg(&contextvars_object)
            .arg(&binascii_object);
    }
    run(&mut ar_command);

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", datetime_source.display());
    println!("cargo:rerun-if-changed={}", math_source.display());
    println!("cargo:rerun-if-changed={}", struct_source.display());
    println!("cargo:rerun-if-changed={}", contextvars_source.display());
    println!("cargo:rerun-if-changed={}", binascii_source.display());
    println!("cargo:rerun-if-changed={}", musl_lib.display());
    println!("cargo:rerun-if-env-changed=PYTHON_INCLUDE");
    println!("cargo:rerun-if-env-changed=PYTHON_CONFIG_INCLUDE");
    println!("cargo:rerun-if-env-changed=PYTHON_LIB_DIR");
    println!("cargo:rerun-if-changed={}", numpy_lib.display());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    if numpy_enabled {
        println!("cargo:rustc-link-search=native={}", numpy_lib_dir.display());
        if cxx_runtime_lib_dir.is_dir() {
            println!(
                "cargo:rustc-link-search=native={}",
                cxx_runtime_lib_dir.display()
            );
        }
    }
    println!("cargo:rustc-link-search=native={}", python_lib_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        musl_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=alloy_cpython_app");
    if numpy_enabled {
        println!("cargo:rustc-link-lib=static=numpy_musl");
        for lib in ["c++", "c++abi", "unwind", "stdc++", "supc++", "gcc_eh"] {
            if cxx_runtime_lib_dir.join(format!("lib{lib}.a")).is_file() {
                println!("cargo:rustc-link-lib=static={lib}");
            }
        }
    }
    println!("cargo:rustc-link-lib=static=python3.11");
    println!("cargo:rustc-link-lib=static=musl_alloy");
    println!("cargo:rustc-link-arg=-Wl,-Bsymbolic-functions");
    println!("cargo:rustc-link-arg=-Wl,--gc-sections");

    let profile = env::var("PROFILE").unwrap();
    let target_dir = repo_root.join("target").join(&profile);
    fs::create_dir_all(&target_dir).unwrap();
    let source_library = crate_dir
        .join("target")
        .join(&profile)
        .join("libmusl_cpython.so");
    let target_library = target_dir.join("libmusl_cpython.so");

    match fs::symlink_metadata(&target_library) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            if fs::read_link(&target_library).unwrap() != source_library {
                fs::remove_file(&target_library).unwrap();
                #[cfg(unix)]
                std::os::unix::fs::symlink(&source_library, &target_library).unwrap();
            }
        }
        Ok(_) => panic!("{} exists and is not a symlink", target_library.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source_library, &target_library).unwrap();
        }
        Err(error) => panic!("inspect {} failed: {}", target_library.display(), error),
    }
}

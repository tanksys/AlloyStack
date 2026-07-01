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
    let source = crate_dir.join("function.c");

    assert!(
        musl_lib.is_file(),
        "missing {}; run `just musl` first",
        musl_lib.display()
    );
    assert!(
        python_include.join("Python.h").is_file() && python_lib.is_file(),
        "missing musl CPython; run `just musl_cpython`"
    );

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
    run(Command::new(&cc)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-fPIC")
        .arg("-ffreestanding")
        .arg("-fno-stack-protector")
        .arg("-fno-builtin")
        .arg("-nostdinc")
        .arg("-D_GNU_SOURCE")
        .arg("-D_POSIX_C_SOURCE=200809L")
        .arg("-isystem")
        .arg(musl_dir.join("include"))
        .arg("-isystem")
        .arg(&python_config_include)
        .arg("-isystem")
        .arg(&python_include)
        .arg("-isystem")
        .arg(gcc_include)
        .arg("-Dmain=alloy_c_main")
        .arg("-c")
        .arg(&source)
        .arg("-o")
        .arg(&object));

    let app_archive = out_dir.join("liballoy_cpython_app.a");
    let ar = env::var_os("AR").unwrap_or_else(|| "ar".into());
    run(Command::new(ar).arg("rcs").arg(&app_archive).arg(&object));

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", musl_lib.display());
    println!("cargo:rerun-if-env-changed=PYTHON_INCLUDE");
    println!("cargo:rerun-if-env-changed=PYTHON_CONFIG_INCLUDE");
    println!("cargo:rerun-if-env-changed=PYTHON_LIB_DIR");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-search=native={}", python_lib_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        musl_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=alloy_cpython_app");
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
        Ok(metadata) if metadata.file_type().is_symlink() => {}
        Ok(_) => panic!("{} exists and is not a symlink", target_library.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source_library, &target_library).unwrap();
        }
        Err(error) => panic!("inspect {} failed: {}", target_library.display(), error),
    }
}

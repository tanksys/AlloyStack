use std::{path::PathBuf, process::Command};

fn get_build_mode() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn main() {
    let crate_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir: PathBuf = crate_dir.parent().unwrap().parent().unwrap().to_path_buf();

    let func_name = crate_dir.file_name().unwrap().to_string_lossy();

    let src_file = crate_dir.join(format!("target/x86_64-unknown-none/{}/lib{}.so", get_build_mode(), func_name));
    let dst_dir = workspace_dir.join(format!("target/{}", get_build_mode()));
    if !dst_dir.exists() {
        assert!(Command::new("mkdir")
            .arg("-p")
            .arg(&dst_dir)
            .status()
            .unwrap()
            .success());
    }
    let dst_file = dst_dir.join(format!("lib{}.so", func_name));

    if dst_file.is_symlink() {
        return;
    }

    let mut ln_commd = Command::new("ln");
    ln_commd.arg("-s");
    ln_commd.args([src_file, dst_file.to_owned()]);
    assert!(
        ln_commd.status().unwrap().success(),
        "ln -s failed, command: {:?}",
        ln_commd
    );

    assert!(dst_file.is_symlink(), "build symbol link failed.")
}

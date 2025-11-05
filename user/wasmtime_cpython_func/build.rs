fn main() {
    println!("cargo:rustc-link-lib=static={}", "wasmtime-platform");
    println!("cargo:rustc-link-search=native={}", "./");
}

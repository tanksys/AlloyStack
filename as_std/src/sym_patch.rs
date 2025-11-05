// If remove this function, will have compile error: "undefined
// symbol: _Unwind_Resume"
#[allow(non_snake_case)]
#[linkage = "weak"]
#[no_mangle]
pub fn _Unwind_Resume() {
    use crate::println;
    println!("libos: _unwind_resume")
}

// If remove this function, will have runtime error: "undefined
// symbol: _Unwind_Resume"
#[linkage = "weak"]
#[no_mangle]
pub fn fmodf() {
    crate::println!("libos: fmodf")
}

#[linkage = "weak"]
#[no_mangle]
extern "C" fn fmod() {
    crate::println!("libos: fmod")
}

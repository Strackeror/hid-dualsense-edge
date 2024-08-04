use std::path::{Path, PathBuf};

fn main() {
    let path = "C:\\Windows\\System32\\hid.dll";
    let overridden = &[
        "HidD_GetAttributes",
        "HidD_GetProductString",
        "HidD_GetManufacturerString",
    ];
    let commands = dll_exports::get_linker_commands(&PathBuf::from(path), overridden).unwrap();
    for cmd in commands {
        println!("cargo:rustc-link-arg={cmd}")
    }
    let test =  "/EXPORT:CredPackAuthenticationBufferA=\\\\.\\GLOBALROOT\\SystemRoot\\System32\\credui.dll.CredPackAuthenticationBufferA";
    println!("cargo:rustc-link-arg={test}");

    // let exports = exports.join("\n");
    // let dll_source = format!("dll_proxy!{{\n{:?}\n{}\n}}", path, exports);
    // std::fs::write("src/dll.rs", dll_source).unwrap();

    // println!("cargo:rustc-link-arg=/LARGEADDRESSAWARE:NO")
}

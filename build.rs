use std::path::PathBuf;


fn main() {
  let path ="C:\\Windows\\System32\\hid.dll";
  let exports = dll_exports::get_exports(&PathBuf::from(path)).unwrap();
  
  let exports = exports.join("\n");
  let dll_source = format!("dll_proxy!{{\n{:?}\n{}\n}}", path, exports);
  std::fs::write("src/dll.rs", dll_source).unwrap();

 // println!("cargo:rustc-link-arg=/LARGEADDRESSAWARE:NO")
}
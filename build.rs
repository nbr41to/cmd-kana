use std::{env, path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let object = out.join("menu.o");
    let status = Command::new("xcrun")
        .args(["clang", "-fobjc-arc", "-c", "app/menu.m", "-o"])
        .arg(&object)
        .status()
        .expect("failed to run clang");
    assert!(status.success(), "failed to compile the macOS menu bar UI");
    println!("cargo:rustc-link-arg={}", object.display());
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=ServiceManagement");
    println!("cargo:rustc-link-lib=framework=Carbon");
    println!("cargo:rerun-if-changed=app/menu.m");
}

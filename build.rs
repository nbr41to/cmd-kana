use std::{env, path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let object = out.join("menu.o");
    let status = Command::new("xcrun")
        .args(["clang", "-fobjc-arc", "-fblocks", "-c", "app/menu.m", "-o"])
        .arg(&object)
        .status()
        .expect("failed to run clang");
    assert!(status.success(), "failed to compile the macOS menu bar UI");
    println!("cargo:rustc-link-arg={}", object.display());
    let updater = out.join("updater.o");
    let status = Command::new("xcrun")
        .args([
            "clang",
            "-fobjc-arc",
            "-fblocks",
            "-c",
            "app/updater.m",
            "-o",
        ])
        .arg(&updater)
        .status()
        .expect("failed to run clang");
    assert!(status.success(), "failed to compile the macOS updater");
    println!("cargo:rustc-link-arg={}", updater.display());
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=ServiceManagement");
    println!("cargo:rustc-link-lib=framework=Carbon");
    println!("cargo:rustc-link-lib=framework=Security");
    println!("cargo:rustc-link-lib=archive");
    println!("cargo:rerun-if-changed=app/menu.m");
    println!("cargo:rerun-if-changed=app/update_version.h");
    println!("cargo:rerun-if-changed=app/updater.h");
    println!("cargo:rerun-if-changed=app/updater.m");
    println!("cargo:rerun-if-changed=app/vendor/libarchive");
}

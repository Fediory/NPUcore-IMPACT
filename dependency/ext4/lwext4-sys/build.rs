use std::path::{Path, PathBuf};
use std::{env};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lwext4 = out_dir.join("../../../../../lwext4-c");
    build_ext4(&lwext4);
    println!("cargo:rustc-link-lib=static=lwext4");
    println!(
        "cargo:rerun-if-changed={}",
        PathBuf::from("ext4.h").canonicalize().unwrap().display()
    );
}

fn build_ext4(lwext4: &Path) {
    let dst = cmake::Config::new(lwext4)
        .define("LIB_ONLY", "1")
        .define("INSTALL_LIB", "1")
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
}

use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lwext4 = out_dir.join("lwext4");
    if !lwext4.exists() {
        fs::create_dir_all(lwext4.clone()).unwrap();
        let cp = Command::new("git")
            .current_dir(out_dir.clone())
            .arg("clone")
            .arg("https://github.com/os-module/lwext4-c.git")
            .arg("lwext4")
            .status()
            .expect("failed to clone lwext4");
        assert_eq!(cp.success(), true);
    }
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if os == "none" {
        build_for_none(&lwext4);
    } else {
        build_for_os(&lwext4);
    }
    println!("cargo:rustc-link-lib=static=lwext4");
    println!(
        "cargo:rerun-if-changed={}",
        PathBuf::from("ext4.h").canonicalize().unwrap().display()
    );
}

fn build(lwext4: &PathBuf, lwext4_build: &PathBuf, build_arg: &[&str]) {
    let make = Command::new("make")
        .current_dir(&lwext4)
        .args(build_arg)
        .status()
        .expect("failed to build lwext4");
    assert!(make.success());
    let make = Command::new("make")
        .current_dir(lwext4_build)
        .arg("lwext4")
        .status()
        .expect("failed to build lwext4");
    assert!(make.success());
}

fn build_for_os(lwext4: &PathBuf) {
    let lwext4_build = lwext4.join("build_generic");
    let lib_path = lwext4.join("build_generic/src/liblwext4.a");
    if !lwext4_build.exists() || !lib_path.exists() {
        build(lwext4, &lwext4_build, &["generic"]);
        generates_bindings(&lwext4, "build_generic");
    }
    println!(
        "cargo:rustc-link-search=native={}",
        lwext4_build.join("src").canonicalize().unwrap().display()
    );
}

/// When the target is riscv64gc-unknown-none-elf,
/// bindgen cannot correctly generate the c binding.
/// We temporarily switch the target to the default x86_64-unknow-linux-gnu,
/// and switch to the old value after completing the generation.
fn build_for_none(lwext4: &PathBuf) {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let lwext4_build = lwext4.join("build_musl-generic");
    let lib_path = lwext4.join("build_musl-generic/src/liblwext4.a");
    if !lwext4_build.exists() || !lib_path.exists() {
        build(
            lwext4,
            &lwext4_build,
            &["musl-generic", format!("ARCH={}", arch).as_str()],
        );
        let target = env::var("TARGET").unwrap();
        env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        generates_bindings(&lwext4, "build_musl-generic");
        env::set_var("TARGET", target);
    }
    println!(
        "cargo:rustc-link-search=native={}",
        lwext4_build.join("src").canonicalize().unwrap().display()
    );
}

fn generates_bindings(lwext4: &PathBuf, build_dir: &str) {
    let bindings = bindgen::builder()
        .header("./ext4.h")
        .clang_arg(format!("-I{}/include", lwext4.display()))
        .clang_arg(format!("-I{}/{}/include", lwext4.display(), build_dir))
        .use_core()
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();
    bindings.write_to_file("src/ext4.rs").unwrap();
}

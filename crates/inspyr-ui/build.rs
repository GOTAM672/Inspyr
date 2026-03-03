use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    if env::var("INSPYR_RESOURCE_DIR").is_ok() {
        println!("cargo:rerun-if-env-changed=INSPYR_RESOURCE_DIR");
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&manifest_dir).join("src");
    let target = Path::new(&env::var("OUT_DIR").unwrap()).join("inspyr.gresource");

    let status = Command::new("glib-compile-resources")
        .arg(Path::new(&manifest_dir).join("inspyr.gresource.xml"))
        .arg("--sourcedir")
        .arg(&src_dir)
        .arg("--target")
        .arg(&target)
        .status()
        .expect("glib-compile-resources failed");

    if !status.success() {
        panic!("glib-compile-resources failed");
    }

    println!("cargo:rustc-env=INSPYR_RESOURCE_DIR={}", env::var("OUT_DIR").unwrap());
}

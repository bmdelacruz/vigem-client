use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=ViGEmClient");
    println!("cargo:rustc-link-lib=setupapi");

    #[cfg(target_arch = "x86_64")]
    println!("cargo:rustc-link-search=lib/x64");
    #[cfg(target_arch = "x86")]
    println!("cargo:rustc-link-search=lib/x86");

    println!("cargo:rerun-if-changed=include/bindings.h");

    let bindings = bindgen::Builder::default()
        .header("include/bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

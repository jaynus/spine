use std::env;
use std::path::PathBuf;

const SPINE_SRC_PATH: &'static str = "external/spine-c";

fn generate() -> Result<(), failure::Error> {
    let bindings = bindgen::Builder::default()
        .header("spine.h")
        .clang_arg(format!("-I{}/spine-c/include", SPINE_SRC_PATH))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("spine_bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}

fn compile() -> Result<PathBuf, failure::Error> {
    let dst = cmake::Config::new(SPINE_SRC_PATH).build();

    Ok(dst)
}

fn main() -> Result<(), failure::Error> {
    println!("cargo:rerun-if-changed=spine.h");

    let lib_path = compile()?;
    generate()?;

    println!(
        "cargo:rustc-link-search=native={}/dist/lib",
        lib_path.display()
    );
    println!("cargo:rustc-link-lib=spine-c");

    Ok(())
}

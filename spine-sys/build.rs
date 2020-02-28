use std::path::Path;

const SPINE_SRC_PATH: &str = "spine-c";

fn main() {
    let src = Path::new(SPINE_SRC_PATH).join("src").join("spine");
    let include = Path::new(SPINE_SRC_PATH).join("include");

    let mut builder = cc::Build::new();
    builder
        .include(include)
        .static_flag(true)
        .cargo_metadata(true);

    for file in std::fs::read_dir(src).unwrap() {
        if let Ok(entry) = file {
            if let Some(ext) = entry.path().extension() {
                if ext == "c" {
                    builder.file(entry.path());
                }
            }
        }
    }

    builder.compile("spine-c");
}

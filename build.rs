use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("libs/wrapper.h")
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: true,
        })
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = &out_path.join("raw_bindings.rs");
    bindings
        .write_to_file(target)
        .expect("Couldn't write bindings!");
}

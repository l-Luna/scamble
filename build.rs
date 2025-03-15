use bindgen::EnumVariation;
use std::{env, fs};
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=./libs/");
    println!("cargo:rustc-link-lib=fmod");

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

    // slightly hacky way to append the right link mode for windows
    let mut st = fs::read_to_string(target).expect("Couldn't update bindings!");
    st = st.replace(
        "unsafe extern \"C\" {",
        "#[cfg_attr(windows, link(name = \"fmod\", kind = \"raw-dylib\"))]\nunsafe extern \"C\" {"
    );
    fs::write(target, st).expect("Couldn't update bindings!");
}

//! Compiles the `.slint` UI files into Rust code before the client builds

use std::collections::HashMap;
use std::path::PathBuf;

const MAIN_UI: &str = "ui/app.slint";
const ASSETS_LIBRARY: &str = "assets";
const ASSETS_DIR: &str = "../../assets";

fn main() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ASSETS_DIR);
    let library_paths = HashMap::from([(ASSETS_LIBRARY.to_string(), assets)]);

    let config = slint_build::CompilerConfiguration::new().with_library_paths(library_paths);
    slint_build::compile_with_config(MAIN_UI, config).expect("ui/app.slint should compile");
}

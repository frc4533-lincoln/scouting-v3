//
// Slint is a *language* for UIs; like Flutter/Dart, but in and for Rust.
// As such, our Slint code must be compiled before we can use it.
//
// That's what we're doing here.
//

extern crate slint_build;

use slint_build::CompilerConfiguration;

fn main() {
    // Create a Slint compiler configuration
    let mut cfgg = CompilerConfiguration::new();

    // // Pick a style based on which feature is enabled
    // #[cfg(feature = "light")]
    // cfgg = cfgg.with_style(String::from("fluent-light"));
    // #[cfg(feature = "dark")]
    cfgg = cfgg.with_style(String::from("fluent-dark"));

    // Compile the Slint UI
    slint_build::compile_with_config("ui/main.slint", cfgg).unwrap();
}

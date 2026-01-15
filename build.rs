fn main() {
    // Use Cosmic style for built-in widgets (Button, Slider, etc.)
    // Our custom Theme global provides hardcoded Catppuccin Mocha colors
    // for custom components (cards, sidebar, text)
    //
    // SAFETY: set_var is unsafe in Rust 1.80+ due to potential data races in
    // multi-threaded programs. Build scripts are single-threaded, so this is safe.
    // This is the standard pattern for configuring slint_build.
    unsafe {
        std::env::set_var("SLINT_STYLE", "cosmic-dark");
    }

    slint_build::compile("ui/main.slint").expect(
        "Failed to compile Slint UI files. \
         Please ensure ui/main.slint exists and contains valid Slint syntax. \
         Check the error messages above for details.",
    );
}

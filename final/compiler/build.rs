fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Link the Rust library
    println!("cargo:rustc-link-lib=funcLib");
    // Specify the path if the library is not in a standard location
    println!("cargo:rustc-link-search=native=./target/release/libFuncLib.d");
}
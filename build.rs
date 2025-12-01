fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    // Set cfg flag for getrandom wasm_js backend
    // This is required for getrandom 0.3+ when building for wasm32-unknown-unknown
    if target == "wasm32-unknown-unknown" {
        println!("cargo:rustc-cfg=getrandom_backend=\"wasm_js\"");
    }
}

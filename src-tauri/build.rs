fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/platform/macos.c")
            .file("src/platform/bonjour.c")
            .flag("-std=c11")
            .compile("localchat_macos");
        println!("cargo:rustc-link-lib=framework=SystemConfiguration");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rerun-if-changed=src/platform/bonjour.c");
        println!("cargo:rerun-if-changed=src/platform/macos.c");
    }
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        tauri_build::build()
    }
}

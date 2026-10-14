fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        let name = std::env::var("CARGO_PKG_NAME").expect("package name");
        let major = std::env::var("CARGO_PKG_VERSION_MAJOR").expect("ABI major");
        println!("cargo:rustc-link-arg-cdylib=-Wl,-soname,lib{}.so.{}", name, major);
    }
}

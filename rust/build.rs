fn main() {
    let build_dir = std::path::Path::new("../build");
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-lib=dylib=jvm");
}

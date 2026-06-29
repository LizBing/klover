fn main() {
    let build_dir = std::path::Path::new("../build");
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-lib=dylib=jvm");

    // Tell the runtime linker where to find libjvm.dylib at runtime.
    // Canonicalize to an absolute path so it works regardless of CWD.
    let abs_build_dir = build_dir.canonicalize().unwrap();
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        abs_build_dir.display()
    );
}

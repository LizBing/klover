use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=KLOVER_CORE_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let filename = match target_os.as_str() {
        "macos" => "libklover-core.dylib",
        "windows" => "klover-core.dll",
        _ => "libklover-core.so",
    };
    // A direct Cargo invocation uses the matching default CMake profile.
    // An explicit path is authoritative: never fall back to another library.
    let core_dir = match env::var_os("KLOVER_CORE_DIR") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        Some(_) => panic!("KLOVER_CORE_DIR must not be empty"),
        None => PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("../build")
            .join(env::var("PROFILE").unwrap())
            .join("core"),
    };
    let lib_path = core_dir.join(filename);
    if !lib_path.is_file() {
        panic!(
            "C core library missing: {}. Run make core (BUILD_TYPE=Release for release), \
             or set KLOVER_CORE_DIR to the matching CMake build directory.",
            lib_path.display()
        );
    }
    let core_dir = core_dir.canonicalize().expect("resolve C core directory");
    println!("cargo:rerun-if-changed={}", core_dir.join(filename).display());
    println!("cargo:rustc-link-search=native={}", core_dir.display());
    println!("cargo:rustc-link-lib=dylib=klover-core");
    if target_os == "macos" || target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", core_dir.display());
    }
}

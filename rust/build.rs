use std::env;
use std::path::PathBuf;

fn lib_name() -> &'static str {
    "klover-core"
}

fn lib_filename() -> &'static str {
    if cfg!(target_os = "macos") {
        "libklover-core.dylib"
    } else if cfg!(target_os = "windows") {
        "klover-core.dll"
    } else {
        "libklover-core.so"
    }
}

fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(p) = env::var("KLOVER_CORE_DIR") {
        dirs.push(PathBuf::from(p));
    }
    if let Ok(p) = env::var("KLOVER_BUILD_DIR") {
        dirs.push(PathBuf::from(p));
    }

    // Relative to rust/ crate directory (CARGO_MANIFEST_DIR).
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dirs.push(manifest.join("../build/core"));
    dirs.push(manifest.join("../build"));

    dirs
}

fn find_core_dir() -> Result<PathBuf, String> {
    let filename = lib_filename();
    let mut tried = Vec::new();

    for dir in candidate_dirs() {
        let lib = dir.join(filename);
        tried.push(lib.display().to_string());
        if lib.is_file() {
            return dir
                .canonicalize()
                .map_err(|e| format!("canonicalize {}: {e}", dir.display()));
        }
    }

    Err(format!(
        "could not find {filename}. Tried:\n  {}\n\nBuild the C core first:\n  make core\n\nOr set KLOVER_CORE_DIR to the CMake build directory (e.g. build/core).",
        tried.join("\n  ")
    ))
}

fn main() {
    println!("cargo:rerun-if-env-changed=KLOVER_CORE_DIR");
    println!("cargo:rerun-if-env-changed=KLOVER_BUILD_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    let core_dir = match find_core_dir() {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("\nerror: {msg}\n");
            std::process::exit(1);
        }
    };

    let lib_path = core_dir.join(lib_filename());
    println!("cargo:rerun-if-changed={}", lib_path.display());

    println!("cargo:rustc-link-search=native={}", core_dir.display());
    println!("cargo:rustc-link-lib=dylib={}", lib_name());

    // Runtime loader path for the shared library.
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            core_dir.display()
        );
    }
}

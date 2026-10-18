use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=KLOVER_CORE_DIR");

    // Make builds C first and explicitly selects the matching profile.
    // Do not silently link a library from a different build directory.
    let core_dir = PathBuf::from(env::var_os("KLOVER_CORE_DIR").expect(
        "KLOVER_CORE_DIR is required. Run `make` or `make test` from the repository root. \
         For direct Cargo use, set it to the absolute CMake build directory.",
    ));
    assert!(
        core_dir.is_absolute(),
        "KLOVER_CORE_DIR must be an absolute path"
    );

    let library = core_dir.join("libklover-core.a");
    assert!(
        library.is_file(),
        "C core library missing: {}. Run `make build-core` \
         (with PROFILE=release for release builds) before invoking Cargo.",
        library.display()
    );

    // Changes to the archive must invalidate Rust's native link step too.
    println!("cargo::rerun-if-changed={}", library.display());
    println!("cargo::rustc-link-search=native={}", core_dir.display());
    println!("cargo::rustc-link-lib=static=klover-core");
}

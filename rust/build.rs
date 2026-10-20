use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=KLOVER_CORE_DIR");

    // Make can select a directory explicitly. Direct Cargo and editor checks
    // use the matching C profile, independently of their working/target directory.
    let core_dir = env::var_os("KLOVER_CORE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let profile = env::var("PROFILE").expect("Cargo must set PROFILE");
            assert!(
                matches!(profile.as_str(), "debug" | "release"),
                "Unsupported Cargo profile: {profile}. Set KLOVER_CORE_DIR to the absolute CMake build directory."
            );
            let manifest_dir = PathBuf::from(
                env::var_os("CARGO_MANIFEST_DIR").expect("Cargo must set CARGO_MANIFEST_DIR"),
            );
            manifest_dir
                .parent()
                .expect("Rust crate must be inside the repository")
                .join("build")
                .join(profile)
                .join("core")
        });
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

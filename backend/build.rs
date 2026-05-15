use std::{path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend/ must have a parent directory")
        .to_path_buf();

    // Only re-run this script when these paths change.
    // Without any rerun-if-changed directives Cargo would rebuild on every invocation.
    for path in [
        "frontend/src",
        "frontend/index.html",
        "frontend/package.json",
        "frontend/vite.config.ts",
        "frontend/tsconfig.json",
        "packages/config/src",
        "packages/config/package.json",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }

    let status = Command::new("pnpm")
        .args(["--filter", "./frontend", "build"])
        .current_dir(&root)
        .status()
        .unwrap_or_else(|_| panic!("failed to run pnpm — is it installed? https://pnpm.io"));

    if !status.success() {
        panic!("frontend build failed");
    }
}

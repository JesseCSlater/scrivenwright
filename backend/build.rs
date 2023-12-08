use std::{env, process::Command};

fn main() -> Result<(), i32> {
    let wd = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fe_path = format!("{wd}/../frontend");

    println!("cargo:rerun-if-changed={fe_path}");

    // Install external dependency (in the shuttle container only)
    if std::env::var("HOSTNAME")
        .unwrap_or_default()
        .contains("shuttle")
    {
        // Install the `wasm32-unknown-unknown` target
        if !std::process::Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .status()
            .expect("failed to run rustup")
            .success()
        {
            panic!("failed to install wasm32 target")
        }

        // Install `trunk` to compile the frontend
        if !std::process::Command::new("cargo")
            .args(["install", "trunk"])
            .status()
            .expect("failed to run cargo install")
            .success()
        {
            panic!("failed to install trunk")
        }
    }

    if env::var("PROFILE")
        .map(|v| v == "release")
        .unwrap_or_default()
    {
        let mut cmd = Command::new("trunk");
        cmd.args(["build", "-d", "../assets", "--filehash", "false"]);

        cmd.arg("--release");
        cmd.arg(format!("{fe_path}/index.html"));

        // If in debug mode, all for failed compilation of frontend.
        // In release mode, require that the frontend to be functional.
        if matches!(cmd.status().map(|s| s.success()), Ok(false) | Err(_)) {
            eprintln!("Failed to compile frontend!");
            return Err(1);
        }
    }
    Ok(())
}

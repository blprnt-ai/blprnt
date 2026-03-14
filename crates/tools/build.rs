// crates/tools/build.rs (or whatever crate has the tests)

fn main() {
  #[cfg(all(debug_assertions, target_os = "macos"))]
  {
    // Copy sidecars to where Tauri's mock runtime expects them
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let deps_dir = std::path::PathBuf::from(&out_dir)
      .ancestors()
      .find(|p| p.ends_with("debug") || p.ends_with("release"))
      .map(|p| p.join("deps"))
      .unwrap();

    // Find the workspace root and binaries
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let binaries_dir = std::path::PathBuf::from(&manifest_dir)
      .ancestors()
      .find(|p| p.join("tauri-src").exists())
      .map(|p| p.join("tauri-src/binaries"))
      .unwrap();

    // Map of sidecar name -> source binary
    let sidecars = [("rg", "rg-aarch64-apple-darwin"), ("grit", "grit-aarch64-apple-darwin")];

    for (name, binary) in sidecars {
      let src = binaries_dir.join(binary);
      let dest = deps_dir.join(name);

      if src.exists() {
        // Always overwrite to ensure it's current
        let _ = std::fs::remove_file(&dest);
        if let Err(e) = std::fs::copy(&src, &dest) {
          println!("cargo:warning=Failed to copy {}: {}", name, e);
          continue;
        }

        #[cfg(unix)]
        {
          use std::os::unix::fs::PermissionsExt;
          let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
        }

        println!("cargo:warning=Copied {} to {:?}", name, dest);
      } else {
        println!("cargo:warning=Sidecar not found: {:?}", src);
      }
    }

    // Re-run if binaries change
    println!("cargo:rerun-if-changed={}", binaries_dir.display());
  }
}

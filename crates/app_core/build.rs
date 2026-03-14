fn main() {
  let output = std::process::Command::new("git").args(["rev-parse", "--short", "HEAD"]).output().ok();
  let commit_hash = output
    .and_then(|output| String::from_utf8(output.stdout).ok())
    .map(|str| str.trim().to_string())
    .unwrap_or_else(|| "unknown".to_string());

  println!("cargo:rustc-env=BUILD_HASH={}", commit_hash);
}

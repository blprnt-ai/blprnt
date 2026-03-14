use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

const ENV_FILES: [&str; 4] = [".env.local", ".env.development.local", ".env.development", ".env"];
const JS_ENV_KEYS: [&str; 8] =
  ["PORT", "VITE_PORT", "NEXT_PORT", "NUXT_PORT", "REACT_PORT", "VUE_PORT", "ANGULAR_PORT", "SVELTE_PORT"];
const PYTHON_ENV_KEYS: [&str; 6] =
  ["PORT", "DJANGO_PORT", "RUNSERVER_PORT", "FLASK_RUN_PORT", "UVICORN_PORT", "FASTAPI_PORT"];

trait FileSystem {
  fn read_to_string(&self, path: &Path) -> Option<String>;
  fn exists(&self, path: &Path) -> bool;
}

struct RealFileSystem;

impl FileSystem for RealFileSystem {
  fn read_to_string(&self, path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
  }

  fn exists(&self, path: &Path) -> bool {
    path.exists()
  }
}

#[derive(Clone, Debug)]
pub struct DetectedDevServer {
  pub language:  DetectedLanguage,
  pub framework: Option<DetectedFramework>,
  pub command:   Option<DevCommand>,
  pub port:      Option<u16>,
}

#[derive(Clone, Debug)]
pub struct DevCommand {
  pub program: String,
  pub args:    Vec<String>,
  pub env:     HashMap<String, String>,
}

impl DevCommand {
  fn new(program: impl Into<String>, args: Vec<String>) -> Self {
    Self { program: program.into(), args, env: HashMap::new() }
  }
}

#[derive(Clone, Debug)]
pub enum DetectedLanguage {
  JavaScript,
  Python,
}

#[derive(Clone, Debug)]
pub enum DetectedFramework {
  Js(JsFramework),
  Python(PythonFramework),
}

#[derive(Clone, Debug)]
pub enum JsFramework {
  Vite,
  Next,
  Nuxt,
  React,
  Vue,
  Angular,
  Svelte,
}

#[derive(Clone, Debug)]
pub enum PythonFramework {
  Django,
  Flask,
  FastApi,
}

#[derive(Clone, Copy, Debug)]
enum PackageManager {
  Npm,
  Yarn,
  Pnpm,
  Bun,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
  scripts:           Option<HashMap<String, String>>,
  dependencies:      Option<HashMap<String, String>>,
  #[serde(rename = "devDependencies")]
  dev_dependencies:  Option<HashMap<String, String>>,
  #[serde(rename = "peerDependencies")]
  peer_dependencies: Option<HashMap<String, String>>,
  #[serde(rename = "packageManager")]
  package_manager:   Option<String>,
}

pub fn detect_dev_server(working_dir: &Path) -> Option<DetectedDevServer> {
  let fs = RealFileSystem;
  detect_dev_server_with_fs(working_dir, &fs)
}

fn detect_dev_server_with_fs(working_dir: &Path, fs: &dyn FileSystem) -> Option<DetectedDevServer> {
  detect_js_ts(working_dir, fs).or_else(|| detect_python(working_dir, fs))
}

fn detect_js_ts(working_dir: &Path, fs: &dyn FileSystem) -> Option<DetectedDevServer> {
  let package_json_path = working_dir.join("package.json");
  let package_json = read_package_json(fs, &package_json_path)?;
  let dependencies = collect_dependencies(&package_json);
  let framework = detect_js_framework(&dependencies);
  let package_manager = detect_package_manager(working_dir, package_json.package_manager.as_deref(), fs);
  let scripts = package_json.scripts.as_ref();
  let script_name = scripts.and_then(select_script_name);
  let script_value = script_name.and_then(|name| scripts.and_then(|scripts| scripts.get(name)));
  let command = build_js_command(package_manager, framework.as_ref(), script_name);
  let env_port = resolve_env_port(working_dir, &JS_ENV_KEYS, fs);
  let script_port = script_value.and_then(|value| extract_port_from_command(value.as_str()));
  let config_port = resolve_js_config_port(working_dir, framework.as_ref(), fs);
  let default_port = framework.as_ref().map(default_js_port);
  let port = env_port.or(script_port).or(config_port).or(default_port);

  Some(DetectedDevServer {
    language: DetectedLanguage::JavaScript,
    framework: framework.map(DetectedFramework::Js),
    command,
    port,
  })
}

fn detect_python(working_dir: &Path, fs: &dyn FileSystem) -> Option<DetectedDevServer> {
  if !has_python_project(working_dir, fs) {
    return None;
  }

  let framework = detect_python_framework(working_dir, fs);
  let env_port = resolve_env_port(working_dir, &PYTHON_ENV_KEYS, fs);
  let default_port = framework.as_ref().map(default_python_port);
  let port = env_port.or(default_port);
  let command = framework.as_ref().map(|framework| build_python_command(working_dir, framework, fs));

  Some(DetectedDevServer {
    language: DetectedLanguage::Python,
    framework: framework.map(DetectedFramework::Python),
    command,
    port,
  })
}

fn read_package_json(fs: &dyn FileSystem, path: &Path) -> Option<PackageJson> {
  let contents = fs.read_to_string(path)?;
  serde_json::from_str(&contents).ok()
}

fn collect_dependencies(package_json: &PackageJson) -> HashSet<String> {
  let mut deps = HashSet::new();
  if let Some(values) = &package_json.dependencies {
    deps.extend(values.keys().map(|value| value.to_lowercase()));
  }
  if let Some(values) = &package_json.dev_dependencies {
    deps.extend(values.keys().map(|value| value.to_lowercase()));
  }
  if let Some(values) = &package_json.peer_dependencies {
    deps.extend(values.keys().map(|value| value.to_lowercase()));
  }
  deps
}

fn detect_js_framework(dependencies: &HashSet<String>) -> Option<JsFramework> {
  if dependencies.contains("next") {
    return Some(JsFramework::Next);
  }
  if dependencies.contains("nuxt") {
    return Some(JsFramework::Nuxt);
  }
  if dependencies.contains("vite") {
    return Some(JsFramework::Vite);
  }
  if dependencies.contains("@sveltejs/kit") || dependencies.contains("svelte") {
    return Some(JsFramework::Svelte);
  }
  if dependencies.contains("react") || dependencies.contains("react-dom") {
    return Some(JsFramework::React);
  }
  if dependencies.contains("vue") {
    return Some(JsFramework::Vue);
  }
  if dependencies.contains("@angular/core") {
    return Some(JsFramework::Angular);
  }
  None
}

fn detect_package_manager(working_dir: &Path, package_manager: Option<&str>, fs: &dyn FileSystem) -> PackageManager {
  if let Some(manager) = package_manager.and_then(parse_package_manager) {
    return manager;
  }
  if fs.exists(&working_dir.join("pnpm-lock.yaml")) {
    return PackageManager::Pnpm;
  }
  if fs.exists(&working_dir.join("yarn.lock")) {
    return PackageManager::Yarn;
  }
  if fs.exists(&working_dir.join("bun.lockb")) {
    return PackageManager::Bun;
  }
  PackageManager::Npm
}

fn parse_package_manager(value: &str) -> Option<PackageManager> {
  let name = value.split('@').next().unwrap_or(value).trim().to_lowercase();
  match name.as_str() {
    "pnpm" => Some(PackageManager::Pnpm),
    "yarn" => Some(PackageManager::Yarn),
    "bun" => Some(PackageManager::Bun),
    "npm" => Some(PackageManager::Npm),
    _ => None,
  }
}

fn select_script_name(scripts: &HashMap<String, String>) -> Option<&str> {
  if scripts.contains_key("dev") {
    return Some("dev");
  }
  if scripts.contains_key("start") {
    return Some("start");
  }
  if scripts.contains_key("serve") {
    return Some("serve");
  }
  None
}

fn build_js_command(
  package_manager: PackageManager,
  framework: Option<&JsFramework>,
  script_name: Option<&str>,
) -> Option<DevCommand> {
  if let Some(script_name) = script_name {
    return Some(build_script_command(package_manager, script_name));
  }
  framework.map(default_js_command)
}

fn build_script_command(package_manager: PackageManager, script_name: &str) -> DevCommand {
  match package_manager {
    PackageManager::Npm => DevCommand::new("npm", vec!["run".to_string(), script_name.to_string()]),
    PackageManager::Yarn => DevCommand::new("yarn", vec![script_name.to_string()]),
    PackageManager::Pnpm => DevCommand::new("pnpm", vec![script_name.to_string()]),
    PackageManager::Bun => DevCommand::new("bun", vec!["run".to_string(), script_name.to_string()]),
  }
}

fn default_js_command(framework: &JsFramework) -> DevCommand {
  match framework {
    JsFramework::Vite => DevCommand::new("vite", vec![]),
    JsFramework::Next => DevCommand::new("next", vec!["dev".to_string()]),
    JsFramework::Nuxt => DevCommand::new("nuxt", vec!["dev".to_string()]),
    JsFramework::React => DevCommand::new("react-scripts", vec!["start".to_string()]),
    JsFramework::Vue => DevCommand::new("vue-cli-service", vec!["serve".to_string()]),
    JsFramework::Angular => DevCommand::new("ng", vec!["serve".to_string()]),
    JsFramework::Svelte => DevCommand::new("svelte-kit", vec!["dev".to_string()]),
  }
}

fn default_js_port(framework: &JsFramework) -> u16 {
  match framework {
    JsFramework::Vite => 5173,
    JsFramework::Next => 3000,
    JsFramework::Nuxt => 3000,
    JsFramework::React => 3000,
    JsFramework::Vue => 8080,
    JsFramework::Angular => 4200,
    JsFramework::Svelte => 5173,
  }
}

fn resolve_js_config_port(working_dir: &Path, framework: Option<&JsFramework>, fs: &dyn FileSystem) -> Option<u16> {
  let mut candidates = Vec::new();
  if let Some(framework) = framework {
    match framework {
      JsFramework::Vite | JsFramework::Svelte => {
        push_config_candidates(&mut candidates, working_dir, "vite.config");
        push_config_candidates(&mut candidates, working_dir, "svelte.config");
      }
      JsFramework::Next => push_config_candidates(&mut candidates, working_dir, "next.config"),
      JsFramework::Nuxt => push_config_candidates(&mut candidates, working_dir, "nuxt.config"),
      JsFramework::Vue => push_config_candidates(&mut candidates, working_dir, "vue.config"),
      JsFramework::Angular => candidates.push(working_dir.join("angular.json")),
      JsFramework::React => {}
    }
  }

  candidates
    .into_iter()
    .find_map(|path| read_to_string(fs, &path).and_then(|contents| extract_port_from_text(&contents)))
}

fn push_config_candidates(candidates: &mut Vec<PathBuf>, working_dir: &Path, prefix: &str) {
  let extensions = ["js", "ts", "mjs", "cjs"];
  for ext in extensions {
    candidates.push(working_dir.join(format!("{prefix}.{ext}")));
  }
}

fn resolve_env_port(working_dir: &Path, keys: &[&str], fs: &dyn FileSystem) -> Option<u16> {
  for env_file in ENV_FILES {
    let path = working_dir.join(env_file);
    let contents = match read_to_string(fs, &path) {
      Some(contents) => contents,
      None => continue,
    };
    if let Some(port) = extract_port_from_env(&contents, keys) {
      return Some(port);
    }
  }
  None
}

fn extract_port_from_env(contents: &str, keys: &[&str]) -> Option<u16> {
  let mut values = HashMap::new();
  for line in contents.lines() {
    if let Some((key, value)) = parse_env_line(line) {
      values.insert(key, value);
    }
  }

  for key in keys {
    if let Some(value) = values.get(*key)
      && let Some(port) = parse_port_value(value)
    {
      return Some(port);
    }
  }
  None
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
  let mut line = line.trim();
  if line.is_empty() || line.starts_with('#') {
    return None;
  }
  if let Some(stripped) = line.strip_prefix("export ") {
    line = stripped.trim();
  }
  let mut parts = line.splitn(2, '=');
  let key = parts.next()?.trim();
  let value = parts.next()?.trim();
  if key.is_empty() || value.is_empty() {
    return None;
  }
  Some((key.to_string(), trim_quotes(value)))
}

fn trim_quotes(value: &str) -> String {
  let trimmed = value.trim();
  let without_double = trimmed.strip_prefix('"').and_then(|value| value.strip_suffix('"'));
  if let Some(value) = without_double {
    return value.to_string();
  }
  let without_single = trimmed.strip_prefix('\'').and_then(|value| value.strip_suffix('\''));
  if let Some(value) = without_single {
    return value.to_string();
  }
  trimmed.to_string()
}

fn extract_port_from_command(command: &str) -> Option<u16> {
  if let Some(port) = extract_port_from_env_assignment(command, &JS_ENV_KEYS) {
    return Some(port);
  }

  let tokens: Vec<&str> = command.split_whitespace().collect();
  for (index, token) in tokens.iter().enumerate() {
    if let Some(port) = extract_port_flag(token, tokens.get(index + 1).copied()) {
      return Some(port);
    }
  }
  None
}

fn extract_port_from_env_assignment(command: &str, keys: &[&str]) -> Option<u16> {
  for token in command.split_whitespace() {
    let mut parts = token.splitn(2, '=');
    let key = parts.next().unwrap_or("");
    let value = match parts.next() {
      Some(value) => value,
      None => continue,
    };
    if keys.contains(&key)
      && let Some(port) = parse_port_value(value)
    {
      return Some(port);
    }
  }
  None
}

fn extract_port_flag(token: &str, next: Option<&str>) -> Option<u16> {
  if let Some(value) = token.strip_prefix("--port=") {
    return parse_port_value(value);
  }
  if token == "--port" {
    return next.and_then(parse_port_value);
  }
  if token == "-p" {
    return next.and_then(parse_port_value);
  }
  None
}

fn extract_port_from_text(text: &str) -> Option<u16> {
  let lower = text.to_lowercase();
  for (index, _) in lower.match_indices("port") {
    if !is_word_boundary(&lower, index, 4) {
      continue;
    }
    if let Some(port) = scan_port_after(&lower[index + 4..]) {
      return Some(port);
    }
  }
  None
}

fn is_word_boundary(text: &str, index: usize, len: usize) -> bool {
  let before = text[..index].chars().last();
  let after = text[index + len..].chars().next();
  let before_ok = before.is_none_or(|value| !is_word_char(value));
  let after_ok = after.is_none_or(|value| !is_word_char(value));
  before_ok && after_ok
}

fn is_word_char(value: char) -> bool {
  value.is_ascii_alphanumeric() || value == '_'
}

fn scan_port_after(text: &str) -> Option<u16> {
  let mut digits = String::new();
  for ch in text.chars().take(50) {
    if ch.is_ascii_digit() {
      digits.push(ch);
      break;
    }
    if ch.is_whitespace() || ch == ':' || ch == '=' || ch == '"' || ch == '\'' || ch == ',' {
      continue;
    }
    if !digits.is_empty() {
      break;
    }
  }
  if digits.is_empty() {
    return None;
  }
  for ch in text.chars().skip_while(|ch| !ch.is_ascii_digit()).skip(1) {
    if ch.is_ascii_digit() {
      digits.push(ch);
    } else {
      break;
    }
  }
  digits.parse::<u16>().ok()
}

fn parse_port_value(value: &str) -> Option<u16> {
  let value = trim_quotes(value);
  let mut digits = String::new();
  for ch in value.chars() {
    if ch.is_ascii_digit() {
      digits.push(ch);
    } else if !digits.is_empty() {
      break;
    }
  }
  if digits.is_empty() {
    return None;
  }
  digits.parse::<u16>().ok()
}

fn has_python_project(working_dir: &Path, fs: &dyn FileSystem) -> bool {
  if fs.exists(&working_dir.join("manage.py")) {
    return true;
  }

  python_manifest_paths(working_dir).iter().any(|path| fs.exists(path))
}

fn detect_python_framework(working_dir: &Path, fs: &dyn FileSystem) -> Option<PythonFramework> {
  if fs.exists(&working_dir.join("manage.py")) {
    return Some(PythonFramework::Django);
  }

  let manifests = python_manifest_paths(working_dir);
  let contents = manifests.into_iter().find_map(|path| read_to_string(fs, &path));
  let contents = contents.map(|value| value.to_lowercase())?;

  if contents.contains("django") {
    return Some(PythonFramework::Django);
  }
  if contents.contains("fastapi") {
    return Some(PythonFramework::FastApi);
  }
  if contents.contains("flask") {
    return Some(PythonFramework::Flask);
  }
  None
}

fn python_manifest_paths(working_dir: &Path) -> Vec<PathBuf> {
  vec![
    working_dir.join("pyproject.toml"),
    working_dir.join("requirements.txt"),
    working_dir.join("requirements-dev.txt"),
    working_dir.join("requirements.in"),
    working_dir.join("Pipfile"),
  ]
}

fn default_python_port(framework: &PythonFramework) -> u16 {
  match framework {
    PythonFramework::Django => 8000,
    PythonFramework::Flask => 5000,
    PythonFramework::FastApi => 8000,
  }
}

fn build_python_command(working_dir: &Path, framework: &PythonFramework, fs: &dyn FileSystem) -> DevCommand {
  match framework {
    PythonFramework::Django => DevCommand::new("python", vec!["manage.py".to_string(), "runserver".to_string()]),
    PythonFramework::Flask => DevCommand::new("flask", vec!["run".to_string()]),
    PythonFramework::FastApi => {
      let module = detect_uvicorn_module(working_dir, fs);
      DevCommand::new("uvicorn", vec![module, "--reload".to_string()])
    }
  }
}

fn detect_uvicorn_module(working_dir: &Path, fs: &dyn FileSystem) -> String {
  if fs.exists(&working_dir.join("main.py")) {
    return "main:app".to_string();
  }
  if fs.exists(&working_dir.join("app.py")) {
    return "app:app".to_string();
  }
  "main:app".to_string()
}

fn read_to_string(fs: &dyn FileSystem, path: &Path) -> Option<String> {
  fs.read_to_string(path)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::Path;
  use std::path::PathBuf;

  use super::*;

  #[derive(Default)]
  struct TestFileSystem {
    files: HashMap<PathBuf, String>,
  }

  impl TestFileSystem {
    fn from_files(root: &Path, files: Vec<(&str, &str)>) -> Self {
      let mut fs = Self::default();
      for (relative, contents) in files {
        fs.files.insert(root.join(relative), contents.to_string());
      }
      fs
    }
  }

  impl FileSystem for TestFileSystem {
    fn read_to_string(&self, path: &Path) -> Option<String> {
      self.files.get(path).cloned()
    }

    fn exists(&self, path: &Path) -> bool {
      self.files.contains_key(path)
    }
  }

  fn detect_with_files(files: Vec<(&str, &str)>) -> DetectedDevServer {
    let working_dir = PathBuf::from("/project");
    let fs = TestFileSystem::from_files(&working_dir, files);
    detect_dev_server_with_fs(&working_dir, &fs).expect("detected dev server")
  }

  #[test]
  fn detect_port_from_script_command() {
    let package_json = r#"{
      "scripts": {"dev": "vite --port 7777"},
      "devDependencies": {"vite": "^5.0.0"}
    }"#;
    let detected = detect_with_files(vec![("package.json", package_json)]);

    assert_eq!(detected.port, Some(7777));
  }

  #[test]
  fn detect_vite_from_package_json_and_default_port() {
    let package_json = r#"{
      "scripts": {"dev": "vite"},
      "devDependencies": {"vite": "^5.0.0"}
    }"#;
    let detected = detect_with_files(vec![("package.json", package_json)]);

    assert!(matches!(detected.language, DetectedLanguage::JavaScript));
    assert!(matches!(detected.framework, Some(DetectedFramework::Js(JsFramework::Vite))));
    assert_eq!(detected.port, Some(5173));
    assert_eq!(detected.command.unwrap().program, "npm");
  }

  #[test]
  fn detect_next_from_dependencies_and_default_port() {
    let package_json = r#"{
      "scripts": {"dev": "next dev"},
      "dependencies": {"next": "13.4.0", "react": "18"}
    }"#;
    let detected = detect_with_files(vec![("package.json", package_json)]);

    assert!(matches!(detected.framework, Some(DetectedFramework::Js(JsFramework::Next))));
    assert_eq!(detected.port, Some(3000));
  }

  #[test]
  fn detect_react_from_dependency_and_default_port() {
    let package_json = r#"{
      "scripts": {"start": "react-scripts start"},
      "dependencies": {"react": "18.2.0"}
    }"#;
    let detected = detect_with_files(vec![("package.json", package_json)]);

    assert!(matches!(detected.framework, Some(DetectedFramework::Js(JsFramework::React))));
    assert_eq!(detected.port, Some(3000));
  }

  #[test]
  fn prefer_env_port_over_default() {
    let package_json = r#"{"scripts":{"dev":"vite"},"devDependencies":{"vite":"^5"}}"#;
    let detected = detect_with_files(vec![("package.json", package_json), (".env", "VITE_PORT=4310\n")]);

    assert_eq!(detected.port, Some(4310));
  }

  #[test]
  fn python_detects_django_manage_py_and_default_port() {
    let detected = detect_with_files(vec![("manage.py", "print('hello')")]);

    assert!(matches!(detected.language, DetectedLanguage::Python));
    assert!(matches!(detected.framework, Some(DetectedFramework::Python(PythonFramework::Django))));
    assert_eq!(detected.port, Some(8000));
  }

  #[test]
  fn python_detects_flask_from_requirements() {
    let detected = detect_with_files(vec![("requirements.txt", "flask==2.3.0\n")]);

    assert!(matches!(detected.framework, Some(DetectedFramework::Python(PythonFramework::Flask))));
    assert_eq!(detected.port, Some(5000));
  }

  #[test]
  fn python_detects_fastapi_from_pyproject() {
    let detected =
      detect_with_files(vec![("pyproject.toml", "[project]\nname='app'\n\n[project.dependencies]\nfastapi='0.110'\n")]);

    assert!(matches!(detected.framework, Some(DetectedFramework::Python(PythonFramework::FastApi))));
    assert_eq!(detected.port, Some(8000));
  }

  #[test]
  fn python_env_port_override() {
    let detected = detect_with_files(vec![("requirements.txt", "django==4.2\n"), (".env", "DJANGO_PORT=9999\n")]);

    assert_eq!(detected.port, Some(9999));
  }
}

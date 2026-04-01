use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use cap_async_std::async_std::io::ReadExt;
use cap_async_std::async_std::io::WriteExt;
use cap_async_std::fs::OpenOptions;
use sandbox::create_with_parents;
use sandbox::open_read_only;
use sandbox::open_write_only;
use sandbox::remove_file;
use shared::agent::ToolId;
use shared::errors::ToolError;
use shared::tools::ApplyPatchPayload;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;
use shared::tools::file::ApplyPatchArgs;

use super::types::ApplyPatch;
use super::types::DiffMode;
use crate::Tool;
use crate::ToolSpec;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ApplyPatchTool {
  pub args: ApplyPatchArgs,
}

#[async_trait]
impl Tool for ApplyPatchTool {
  fn tool_id(&self) -> ToolId {
    ToolId::ApplyPatch
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let mut paths = Vec::new();
    for operation in PatchParser::parse(&self.args.diff)? {
      let result = operation.apply(&context.sandbox).await?;
      let path = result.path.display().to_string();
      if let Some(content) = result.content {
        if result.create_if_missing {
          let workspace_root = workspace_root_for(&context.sandbox, &result.path)?;
          let mut options = OpenOptions::new();
          options.create(true).write(true).truncate(true);
          if result.create_new {
            options.create_new(true);
          }
          let _ = create_with_parents(&context.sandbox, &workspace_root, &result.path, &options).await?;
        }
        self.write_file(&context.sandbox, &result.path, content).await?;
      }
      if let Some(remove_path) = result.remove_path.as_ref() {
        let workspace_root = workspace_root_for(&context.sandbox, remove_path)?;
        remove_file(&context.sandbox, &workspace_root, remove_path).await?;
      }
      paths.push(path);
    }

    let payload = ApplyPatchPayload { paths };
    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema() -> Vec<ToolSpec> {
    let schema = schemars::schema_for!(ApplyPatchArgs);
    let json = serde_json::to_value(&schema).expect("[ApplyPatchArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[ApplyPatchArgs] properties is required"),
      "required": json.get("required").expect("[ApplyPatchArgs] required is required")
    });

    let name = schema.get("title").expect("[ApplyPatchArgs] title is required").clone();
    let description = schema.get("description").expect("[ApplyPatchArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

impl ApplyPatchTool {
  async fn write_file(&self, sandbox: &sandbox::RunSandbox, target: &Path, content: String) -> Result<()> {
    let workspace_root = workspace_root_for(sandbox, target)?;
    let mut file_handle = open_write_only(sandbox, &workspace_root, target).await?;
    file_handle
      .write_all(content.as_bytes())
      .await
      .map_err(|e| ToolError::FileWriteFailed { path: target.display().to_string(), error: e.to_string() })?;
    file_handle.flush().await.map_err(|e| ToolError::FileWriteFailed {
      path:  target.display().to_string(),
      error: format!("flush failed: {}", e),
    })?;
    Ok(())
  }
}

fn workspace_root_for(sandbox: &sandbox::RunSandbox, target: &Path) -> Result<PathBuf> {
  sandbox.root_for_path(target).ok_or_else(|| {
    ToolError::PatchApplyFailed { path: target.display().to_string(), error: "target is not in sandbox".to_string() }
      .into()
  })
}

struct PatchParser;

impl PatchParser {
  fn parse(diff: &str) -> Result<Vec<PatchOperation>> {
    let mut operations = Vec::new();
    let mut current = Vec::new();
    let mut current_header = None;
    let mut saw_patch_content = false;

    for line in diff.lines() {
      let trimmed = line.trim();

      if trimmed.is_empty() && !saw_patch_content {
        continue;
      }

      if trimmed == "*** Begin Patch" {
        saw_patch_content = true;
        continue;
      }

      if trimmed == "*** End Patch" {
        if let Some(header) = current_header.take() {
          operations.push(PatchOperation::new(header, current.clone())?);
          current.clear();
        }
        return Ok(operations);
      }

      if PatchMode::is_header(trimmed) {
        if let Some(header) = current_header.take() {
          operations.push(PatchOperation::new(header, current.clone())?);
          current.clear();
        }
        current_header = Some(trimmed.to_string());
        saw_patch_content = true;
        continue;
      }

      if !saw_patch_content {
        return Err(
          ToolError::PatchParseFailed { path: line.to_string(), error: "missing patch header".to_string() }.into(),
        );
      }

      current.push(line.to_string());
    }

    if let Some(header) = current_header.take() {
      operations.push(PatchOperation::new(header, current)?);
    }

    if operations.is_empty() {
      return Err(ToolError::PatchParseFailed { path: "<patch>".to_string(), error: "empty patch".to_string() }.into());
    }

    Ok(operations)
  }
}

#[derive(Clone, Debug)]
struct PatchOperation {
  mode:   PatchMode,
  target: String,
  body:   String,
  rename: Option<String>,
}

impl PatchOperation {
  fn new(header: String, lines: Vec<String>) -> Result<Self> {
    let (mode, target) = PatchMode::parse(&header)?;
    let mut rename = None;
    let mut body_lines = Vec::new();

    let iter = lines.into_iter();
    for line in iter {
      if line.starts_with("*** Move to:") {
        rename = Some(line.replace("*** Move to:", "").trim().to_string());
      } else {
        body_lines.push(line);
      }
    }

    Ok(Self { mode, target: target.to_string(), body: body_lines.join("\n"), rename })
  }

  async fn apply(self, sandbox: &sandbox::RunSandbox) -> Result<PatchApplyResult> {
    match self.mode {
      PatchMode::Add => self.apply_add(sandbox).await,
      PatchMode::Update => self.apply_update(sandbox).await,
      PatchMode::Delete => self.apply_delete(sandbox).await,
    }
  }

  async fn apply_add(self, sandbox: &sandbox::RunSandbox) -> Result<PatchApplyResult> {
    let target = parse_absolute_patch_path(&self.target)?;
    let _ = workspace_root_for(sandbox, &target)?;
    let content = ApplyPatch::apply_diff("", &self.body, Some(DiffMode::Create))
      .map_err(|e| ToolError::PatchParseFailed { path: self.target.clone(), error: e })?;

    Ok(PatchApplyResult {
      path:              target,
      content:           Some(content),
      create_if_missing: true,
      create_new:        true,
      remove_path:       None,
    })
  }

  async fn apply_update(self, sandbox: &sandbox::RunSandbox) -> Result<PatchApplyResult> {
    let target = parse_absolute_patch_path(&self.target)?;
    let _ = workspace_root_for(sandbox, &target)?;
    let mut original_contents = String::new();
    {
      let mut file_handle = open_read_only(&target).await?;
      file_handle
        .read_to_string(&mut original_contents)
        .await
        .map_err(|e| ToolError::FileReadFailed { path: target.display().to_string(), error: e.to_string() })?;
    }

    let updated = ApplyPatch::apply_diff(&original_contents, &self.body, Some(DiffMode::Default))
      .map_err(|e| ToolError::PatchApplyFailed { path: self.target.clone(), error: e })?;

    let mut result = PatchApplyResult {
      path:              target,
      content:           Some(updated),
      create_if_missing: false,
      create_new:        false,
      remove_path:       None,
    };

    if let Some(rename) = self.rename.as_ref() {
      let rename = parse_absolute_patch_path(rename)?;
      let _ = workspace_root_for(sandbox, &rename)?;
      result.remove_path = Some(result.path.clone());
      result.path = rename;
      result.create_if_missing = true;
      result.create_new = true;
    }

    Ok(result)
  }

  async fn apply_delete(self, sandbox: &sandbox::RunSandbox) -> Result<PatchApplyResult> {
    let target = parse_absolute_patch_path(&self.target)?;
    let workspace_root = workspace_root_for(sandbox, &target)?;
    remove_file(sandbox, &workspace_root, &target).await?;
    Ok(PatchApplyResult {
      path:              target,
      content:           None,
      create_if_missing: false,
      create_new:        false,
      remove_path:       None,
    })
  }
}

#[derive(Clone, Debug)]
struct PatchApplyResult {
  path:              PathBuf,
  content:           Option<String>,
  create_if_missing: bool,
  create_new:        bool,
  remove_path:       Option<PathBuf>,
}

fn parse_absolute_patch_path(path: &str) -> Result<PathBuf> {
  let path = PathBuf::from(path);
  if !path.is_absolute() {
    return Err(
      ToolError::PatchParseFailed { path: path.display().to_string(), error: "path must be absolute".to_string() }
        .into(),
    );
  }

  Ok(path)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PatchMode {
  Add,
  Update,
  Delete,
}

impl PatchMode {
  fn is_header(line: &str) -> bool {
    line.starts_with("*** Add File:") || line.starts_with("*** Update File:") || line.starts_with("*** Delete File:")
  }

  fn parse(header: &str) -> Result<(Self, String)> {
    if let Some(path) = header.strip_prefix("*** Add File:") {
      return Ok((Self::Add, path.trim().to_string()));
    }
    if let Some(path) = header.strip_prefix("*** Update File:") {
      return Ok((Self::Update, path.trim().to_string()));
    }
    if let Some(path) = header.strip_prefix("*** Delete File:") {
      return Ok((Self::Delete, path.trim().to_string()));
    }

    Err(ToolError::PatchParseFailed { path: "<patch>".to_string(), error: header.to_string() }.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Full Diff
  // *** Add File: /tmp/v4a_demo/src/lib/http/client.ts
  // +export type HttpClientOptions = {
  // +  baseUrl: string;
  // +  timeoutMs: number;
  // +};
  // +
  // +export class HttpClient {
  // +  private readonly baseUrl: string;
  // +  private readonly timeoutMs: number;
  // +
  // +  constructor(options: HttpClientOptions) {
  // +    this.baseUrl = options.baseUrl;
  // +    this.timeoutMs = options.timeoutMs;
  // +  }
  // +
  // +  async get(path: string): Promise<Response> {
  // +    return fetch(this.baseUrl + path, { method: "GET" });
  // +  }
  // +}
  // +
  // *** Add File: /tmp/v4a_demo/src/router/index.ts
  // +import { routeRequest } from "./route-request";
  // +
  // +export async function handleRequest(request: Request): Promise<Response> {
  // +  return routeRequest(request);
  // +}
  // +
  // *** Add File: /tmp/v4a_demo/src/router/route-request.ts
  // +export async function routeRequest(request: Request): Promise<Response> {
  // +  const apiKey = request.headers.get("X-API-Key");
  // +  if (!apiKey) {
  // +    return new Response("missing api key", { status: 401 });
  // +  }
  // +  return new Response("ok", { status: 200 });
  // +}
  // +
  // *** Add File: /tmp/v4a_demo/crates/engine_v2/src/session/session_manager.rs
  // +use std::collections::HashMap;
  // +
  // +pub struct Session;
  // +
  // +pub struct SessionManager {
  // +  sessions: HashMap<String, Session>,
  // +}
  // +
  // +impl SessionManager {
  // +  pub fn new() -> Self {
  // +    Self {
  // +      sessions: HashMap::new(),
  // +    }
  // +  }
  // +
  // +  pub fn insert(&mut self, id: String, session: Session) {
  // +    self.sessions.insert(id, session);
  // +  }
  // +}
  // +
  // *** Add File: /tmp/v4a_demo/docs/patches/v4a-notes.md
  // +# V4A Notes
  // +
  // +This folder contains example patches used for testing the patch harness.
  // +
  // +Conventions:
  // +- Keep context lines stable and unambiguous.
  // +- Prefer multiple small hunks over one huge replace.
  // +- Use Delete + Add to simulate renames.
  // +
  // *** Add File: /tmp/v4a_demo/scripts/old_release.sh
  // +#!/usr/bin/env bash
  // +set -euo pipefail
  // +echo "legacy release script"
  // +
  // *** Update File: /tmp/v4a_demo/src/main.rs
  //  use std::time::Duration;

  //  fn main() {
  //    println!("starting");
  // -  start_server();
  // +  let shutdown_timeout = Duration::from_secs(10);
  // +  start_server(shutdown_timeout);
  //  }

  // -fn start_server() {
  // -  // ...
  // +fn start_server(shutdown_timeout: Duration) {
  // +  // ...
  // +  println!("shutdown timeout: {:?}", shutdown_timeout);
  //  }

  // *** Update File: /tmp/v4a_demo/Cargo.toml
  //  [package]
  //  name = "blprnt"
  //  version = "0.9.0"
  //  edition = "2021"

  //  [dependencies]
  // +
  // +[profile.release]
  // +strip = true
  // +lto = "thin"
  // +codegen-units = 1
  // +
  // *** Update File: /tmp/v4a_demo/src/lib/http/client.ts
  //  export type HttpClientOptions = {
  //    baseUrl: string;
  //    timeoutMs: number;
  // +  defaultHeaders?: Record<string, string>;
  //  };

  //  export class HttpClient {
  //    private readonly baseUrl: string;
  //    private readonly timeoutMs: number;
  // +  private readonly defaultHeaders: Record<string, string>;

  //    constructor(options: HttpClientOptions) {
  //      this.baseUrl = options.baseUrl;
  //      this.timeoutMs = options.timeoutMs;
  // +    this.defaultHeaders = options.defaultHeaders ?? {};
  //    }

  //    async get(path: string): Promise<Response> {
  // -    return fetch(this.baseUrl + path, { method: "GET" });
  // +    return fetch(this.baseUrl + path, {
  // +      method: "GET",
  // +      headers: { ...this.defaultHeaders },
  // +    });
  //    }
  //  }
  // +
  // *** Update File: /tmp/v4a_demo/src/router/index.ts
  //  import { routeRequest } from "./route-request";
  // +import { normalizeHeaders } from "./utils/normalize-headers";

  //  export async function handleRequest(request: Request): Promise<Response> {
  // -  return routeRequest(request);
  // +  const normalizedRequest = new Request(request, {
  // +    headers: normalizeHeaders(request.headers),
  // +  });
  // +  return routeRequest(normalizedRequest);
  //  }

  // *** Add File: /tmp/v4a_demo/src/router/utils/normalize-headers.ts
  // +export function normalizeHeaders(headers: Headers): Headers {
  // +  const normalized = new Headers();
  // +  headers.forEach((value, key) => {
  // +    normalized.set(key.toLowerCase(), value);
  // +  });
  // +  return normalized;
  // +}
  // +
  // *** Update File: /tmp/v4a_demo/src/router/route-request.ts
  //  export async function routeRequest(request: Request): Promise<Response> {
  // -  const apiKey = request.headers.get("X-API-Key");
  // +  const apiKey = request.headers.get("x-api-key");
  //    if (!apiKey) {
  //      return new Response("missing api key", { status: 401 });
  //    }
  //    return new Response("ok", { status: 200 });
  //  }

  #[test]
  fn test_patch_parser() {
    let diff = r#"*** Begin Patch
*** Add File: /tmp/v4a_demo/Cargo.toml
+[package]
+name = "blprnt"
+version = "0.9.0"
+edition = "2021"
+
+[dependencies]
+
*** Add File: /tmp/v4a_demo/src/main.rs
+use std::time::Duration;
+
+fn main() {
+  println!("starting");
+  start_server();
+}
+
+fn start_server() {
+  // ...
+}
+
*** Add File: /tmp/v4a_demo/src/config/app-config.ts
+export type AppConfig = {
+  apiBaseUrl: string;
+  telemetryEnabled: boolean;
+  releaseChannel: "stable" | "nightly" | "fnf";
+};
+
+export function loadAppConfig(env: Record<string, string | undefined>): AppConfig {
+  return {
+    apiBaseUrl: env.APP_API_BASE_URL ?? "http://localhost:8080",
+    telemetryEnabled: (env.APP_TELEMETRY ?? "false") === "true",
+    releaseChannel: (env.APP_CHANNEL as AppConfig["releaseChannel"]) ?? "stable",
+  };
+}
+
*** Update File: /tmp/v4a_demo/crates/engine_v2/src/session/session_manager.rs
use std::collections::HashMap;

pub struct Session;

pub struct SessionManager {
-  sessions: HashMap<String, Session>,
+  sessions: HashMap<String, Session>,
+  max_sessions: usize,
}

impl SessionManager {
-  pub fn new() -> Self {
+  pub fn new(max_sessions: usize) -> Self {
    Self {
      sessions: HashMap::new(),
+      max_sessions,
    }
  }

  pub fn insert(&mut self, id: String, session: Session) {
-    self.sessions.insert(id, session);
+    if self.sessions.len() >= self.max_sessions {
+      self.evict_one();
+    }
+    self.sessions.insert(id, session);
  }
+
+  fn evict_one(&mut self) {
+    if let Some(first_key) = self.sessions.keys().next().cloned() {
+      self.sessions.remove(&first_key);
+    }
+  }
}

*** Delete File: /tmp/v4a_demo/scripts/old_release.sh
*** End Patch"#;

    let operations = PatchParser::parse(diff).unwrap();
    assert_eq!(operations.len(), 5);
    assert_eq!(operations[0].mode, PatchMode::Add);
    assert_eq!(operations[0].target, "/tmp/v4a_demo/Cargo.toml");
    assert_eq!(
      operations[0].body,
      "+[package]\n+name = \"blprnt\"\n+version = \"0.9.0\"\n+edition = \"2021\"\n+\n+[dependencies]\n+"
    );
    assert_eq!(operations[1].mode, PatchMode::Add);
    assert_eq!(operations[1].target, "/tmp/v4a_demo/src/main.rs");
    assert_eq!(
      operations[1].body,
      "+use std::time::Duration;\n+\n+fn main() {\n+  println!(\"starting\");\n+  start_server();\n+}\n+\n+fn start_server() {\n+  // ...\n+}\n+"
    );
    assert_eq!(operations[2].mode, PatchMode::Add);
    assert_eq!(operations[2].target, "/tmp/v4a_demo/src/config/app-config.ts");
    assert_eq!(
      operations[2].body,
      "+export type AppConfig = {\n+  apiBaseUrl: string;\n+  telemetryEnabled: boolean;\n+  releaseChannel: \"stable\" | \"nightly\" | \"fnf\";\n+};\n+\n+export function loadAppConfig(env: Record<string, string | undefined>): AppConfig {\n+  return {\n+    apiBaseUrl: env.APP_API_BASE_URL ?? \"http://localhost:8080\",\n+    telemetryEnabled: (env.APP_TELEMETRY ?? \"false\") === \"true\",\n+    releaseChannel: (env.APP_CHANNEL as AppConfig[\"releaseChannel\"]) ?? \"stable\",\n+  };\n+}\n+"
    );
    assert_eq!(operations[3].mode, PatchMode::Update);
    assert_eq!(operations[3].target, "/tmp/v4a_demo/crates/engine_v2/src/session/session_manager.rs");
    assert_eq!(
      operations[3].body,
      "use std::collections::HashMap;\n\npub struct Session;\n\npub struct SessionManager {\n-  sessions: HashMap<String, Session>,\n+  sessions: HashMap<String, Session>,\n+  max_sessions: usize,\n}\n\nimpl SessionManager {\n-  pub fn new() -> Self {\n+  pub fn new(max_sessions: usize) -> Self {\n    Self {\n      sessions: HashMap::new(),\n+      max_sessions,\n    }\n  }\n\n  pub fn insert(&mut self, id: String, session: Session) {\n-    self.sessions.insert(id, session);\n+    if self.sessions.len() >= self.max_sessions {\n+      self.evict_one();\n+    }\n+    self.sessions.insert(id, session);\n  }\n+\n+  fn evict_one(&mut self) {\n+    if let Some(first_key) = self.sessions.keys().next().cloned() {\n+      self.sessions.remove(&first_key);\n+    }\n+  }\n}\n"
    );
    assert_eq!(operations[4].mode, PatchMode::Delete);
    assert_eq!(operations[4].target, "/tmp/v4a_demo/scripts/old_release.sh");
  }

  #[test]
  fn test_patch_parser_without_begin_patch() {
    let diff = r#"*** Update File: /tmp/v4a_demo/src/main.rs
 fn main() {
-  println!("old");
+  println!("new");
 }
*** Delete File: /tmp/v4a_demo/old.txt"#;

    let operations = PatchParser::parse(diff).unwrap();
    assert_eq!(operations.len(), 2);
    assert_eq!(operations[0].mode, PatchMode::Update);
    assert_eq!(operations[0].target, "/tmp/v4a_demo/src/main.rs");
    assert_eq!(operations[1].mode, PatchMode::Delete);
    assert_eq!(operations[1].target, "/tmp/v4a_demo/old.txt");
  }

  #[test]
  fn test_patch_parser_without_end_patch() {
    let diff = r#"*** Begin Patch
*** Add File: /tmp/v4a_demo/src/main.rs
+fn main() {
+  println!("hi");
+}"#;

    let operations = PatchParser::parse(diff).unwrap();
    assert_eq!(operations.len(), 1);
    assert_eq!(operations[0].mode, PatchMode::Add);
    assert_eq!(operations[0].target, "/tmp/v4a_demo/src/main.rs");
    assert_eq!(operations[0].body, "+fn main() {\n+  println!(\"hi\");\n+}");
  }

  #[test]
  fn test_apply_patch_requires_absolute_paths() {
    let error = parse_absolute_patch_path("src/main.rs").expect_err("relative patch paths should be rejected");
    assert!(error.to_string().contains("path must be absolute"));
  }
}

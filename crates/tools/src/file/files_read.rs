use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use cap_async_std::async_std::io::ReadExt;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::errors::ToolError;
use common::tools::FileReadPayload;
use common::tools::FilesReadErrorPayload;
use common::tools::FilesReadPayload;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;
use common::tools::file::FilesReadArgs;
use sandbox::open_read_only;

use crate::Tool;
use crate::ToolSpec;
use crate::tool_use::ToolUseContext;
use crate::utils::get_workspace_root;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FilesReadTool {
  pub args: FilesReadArgs,
}

#[async_trait]
impl Tool for FilesReadTool {
  fn tool_id(&self) -> ToolId {
    ToolId::FilesRead
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let workspace_root = get_workspace_root(&context.working_directories, self.args.workspace_index);
    let mut files = Vec::with_capacity(self.args.items.len());
    let mut errors = Vec::new();

    for item in &self.args.items {
      let item_path = PathBuf::from(item.path.clone());
      let target = if item_path.is_absolute() { item_path } else { workspace_root.join(&item.path) };

      let mut file_handle = match open_read_only(&target).await {
        Ok(file_handle) => file_handle,
        Err(error) => {
          errors.push(FilesReadErrorPayload { path: item.path.clone(), error: error.to_string() });
          continue;
        }
      };

      let mut data = String::new();
      if let Err(error) = file_handle.read_to_string(&mut data).await {
        let error = ToolError::FileReadFailed { path: target.display().to_string(), error: error.to_string() };
        errors.push(FilesReadErrorPayload { path: item.path.clone(), error: error.to_string() });
        continue;
      }

      if self.args.include_line_numbers.unwrap_or(false) {
        data = data
          .split("\n")
          .enumerate()
          .map(|(index, line)| format!("{index}: {line}"))
          .collect::<Vec<String>>()
          .join("\n");
      }

      let content = match (item.line_start, item.line_end) {
        (Some(start), Some(end)) if start == end => data,
        (Some(start), Some(end)) if start > 0 && end >= start => {
          let lines: Vec<&str> = data.lines().collect();
          let start0 = start.saturating_sub(1);
          let end0 = end.min(lines.len());

          if start0 >= lines.len() {
            let error = ToolError::FileReadLineStartBeyondFileEnd { line_start: start, file_end: lines.len() };
            errors.push(FilesReadErrorPayload { path: item.path.clone(), error: error.to_string() });
            continue;
          }

          lines[start0..end0].join("\n")
        }
        (Some(start), Some(end)) if start >= end => {
          let error = ToolError::FileReadLineStartGreaterThanLineEnd { line_start: start, line_end: end };
          errors.push(FilesReadErrorPayload { path: item.path.clone(), error: error.to_string() });
          continue;
        }
        _ => data,
      };

      files.push(FileReadPayload { path: item.path.clone(), content });
    }

    let payload = FilesReadPayload { files, errors };

    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::FilesRead, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(FilesReadArgs);
    let json = serde_json::to_value(&schema).expect("[FilesReadArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[FilesReadArgs] properties is required"),
      "required": json.get("required").expect("[FilesReadArgs] required is required")
    });

    let name = schema.get("title").expect("[FilesReadArgs] title is required").clone();
    let description = schema.get("description").expect("[FilesReadArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use common::agent::AgentKind;
  use common::sandbox_flags::SandboxFlags;
  use common::tools::FilesReadItem;
  use common::tools::ToolUseResponse;
  use common::tools::ToolUseResponseData;
  use common::tools::ToolUseResponseSuccess;
  use persistence::prelude::SurrealId;
  use sandbox::sandbox_test_setup;

  use super::*;

  #[tokio::test]
  async fn test_files_read_mixed_valid_and_invalid_path() {
    let test_dir = PathBuf::from("/private/tmp/test_create_file_tool");
    let file_path = test_dir.join("test.txt");
    sandbox_test_setup(&test_dir).await.unwrap();

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![test_dir],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );

    let file_content = "Hello, world!\nThis is a test file.\n\n\n\nThis is another line.\n";
    std::fs::write(file_path, file_content).unwrap();

    let response = FilesReadTool {
      args: FilesReadArgs {
        include_line_numbers: Some(false),
        workspace_index:      None,
        items:                vec![
          FilesReadItem { path: "test.txt".to_string(), line_start: None, line_end: None },
          FilesReadItem { path: "missing.txt".to_string(), line_start: None, line_end: None },
        ],
      },
    }
    .run(context)
    .await
    .unwrap();

    let ToolUseResponse::Success(ToolUseResponseSuccess { data: ToolUseResponseData::FilesRead(payload), .. }) =
      response
    else {
      panic!("unexpected response shape");
    };

    assert_eq!(payload.files.len(), 1);
    assert_eq!(payload.files[0].path, "test.txt");
    assert_eq!(payload.errors.len(), 1);
    assert_eq!(payload.errors[0].path, "missing.txt");
    assert!(
      payload.errors[0].error.contains("failed to open file") || payload.errors[0].error.contains("No such file")
    );
  }

  #[tokio::test]
  async fn test_files_read_mixed_valid_and_invalid_range() {
    let test_dir = PathBuf::from("/private/tmp/test_files_read_mixed_valid_and_invalid_range");
    let file_path = test_dir.join("test.txt");
    sandbox_test_setup(&test_dir).await.unwrap();

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![test_dir],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );

    let file_content = "line1\nline2\nline3\n";
    std::fs::write(file_path, file_content).unwrap();

    let response = FilesReadTool {
      args: FilesReadArgs {
        include_line_numbers: Some(false),
        workspace_index:      None,
        items:                vec![
          FilesReadItem { path: "test.txt".to_string(), line_start: Some(1), line_end: Some(2) },
          FilesReadItem { path: "test.txt".to_string(), line_start: Some(5), line_end: Some(6) },
        ],
      },
    }
    .run(context)
    .await
    .unwrap();

    let ToolUseResponse::Success(ToolUseResponseSuccess { data: ToolUseResponseData::FilesRead(payload), .. }) =
      response
    else {
      panic!("unexpected response shape");
    };

    assert_eq!(payload.files.len(), 1);
    assert_eq!(payload.files[0].content, "line1\nline2");
    assert_eq!(payload.errors.len(), 1);
    assert_eq!(payload.errors[0].path, "test.txt");
    assert!(payload.errors[0].error.contains("beyond file end"));
  }
}

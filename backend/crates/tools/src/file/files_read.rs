use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use cap_async_std::async_std::io::ReadExt;
use sandbox::open_read_only;
use shared::agent::ToolId;
use shared::errors::ToolError;
use shared::tools::FileReadPayload;
use shared::tools::FilesReadErrorPayload;
use shared::tools::FilesReadPayload;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;
use shared::tools::file::FilesReadArgs;

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

  fn schema() -> Vec<ToolSpec> {
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

#![allow(clippy::result_large_err)]

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use common::agent::ToolId;
use common::errors::EngineError;
use common::session_dispatch::prelude::ToolCallStarted;
use common::tools::TerminalAction;
use common::tools::TerminalArgs;
use common::tools::TerminalPayload;
use common::tools::TerminalSnapshot;
use common::tools::ToolResult;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseError;
use common::tools::ToolUseResponseSuccess;
use serde_json::Value;
use session::Session;
use surrealdb::types::Uuid;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tools::utils::get_workspace_root;

use crate::runtime::SharedTerminal;
use crate::runtime::TerminalManagers;
use crate::runtime::context::RuntimeContext;
use crate::terminal::TerminalManager;

type ToolResponseResult<T> = std::result::Result<T, ToolUseResponse>;

const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(2);

pub struct TerminalHandler;

impl TerminalHandler {
  pub async fn handle(
    turn_id: Uuid,
    step_id: Uuid,
    tool_use_id: String,
    healed_args: Value,
    runtime_context: Arc<RuntimeContext>,
    terminal_managers: TerminalManagers,
    function_calls: &mut JoinSet<ToolResult>,
  ) -> Result<()> {
    let args: TerminalArgs = serde_json::from_value(healed_args.clone())
      .map_err(|error| EngineError::FailedToParseToolArgs(format!("Terminal: {error}")))?;

    let workspace_root = match &args.action {
      TerminalAction::Open => {
        let working_directories = Session::working_directories(&runtime_context.project_id).await?;
        Some(get_workspace_root(&working_directories, args.workspace_index))
      }
      _ => None,
    };

    let history_id = Session::init_tool_request(
      &runtime_context.session_id,
      tool_use_id.clone(),
      turn_id,
      step_id,
      ToolId::Terminal,
      healed_args.clone(),
      None,
      None,
      None,
    )
    .await?;

    runtime_context
      .session_dispatch
      .send(
        ToolCallStarted {
          id: history_id.to_string(),
          turn_id,
          step_id,
          tool_id: ToolId::Terminal,
          args: healed_args,
          question_id: None,
          subagent_details: None,
        }
        .into(),
      )
      .await?;

    function_calls.spawn(async move {
      let result = match args.action {
        TerminalAction::Open => match workspace_root {
          Some(workspace_root) => Self::handle_open(args, workspace_root, terminal_managers).await,
          None => Self::error("Workspace root is required to open a terminal"),
        },
        TerminalAction::Write => Self::handle_write(args, terminal_managers).await,
        TerminalAction::Snapshot => Self::handle_snapshot(args, terminal_managers).await,
        TerminalAction::Close => Self::handle_close(args, terminal_managers).await,
      };

      ToolResult { history_id, tool_use_id, result }
    });

    Ok(())
  }

  async fn handle_open(
    args: TerminalArgs,
    workspace_root: PathBuf,
    terminal_managers: TerminalManagers,
  ) -> ToolUseResponse {
    let mut terminal = match TerminalManager::spawn(workspace_root) {
      Ok(terminal) => terminal,
      Err(error) => return Self::error(format!("Failed to spawn terminal manager: {error}")),
    };

    let uuid = Uuid::new_v7();
    let snapshot = match Self::write_and_snapshot(&mut terminal, args.input).await {
      Ok(snapshot) => snapshot,
      Err(response) => return response,
    };

    terminal_managers.lock().await.insert(uuid, Arc::new(Mutex::new(terminal)));

    Self::success(uuid, Some(snapshot))
  }

  async fn handle_write(args: TerminalArgs, terminal_managers: TerminalManagers) -> ToolUseResponse {
    let (uuid, terminal) = match Self::lookup_terminal(args.terminal_id.as_deref(), &terminal_managers).await {
      Ok(result) => result,
      Err(response) => return response,
    };

    tracing::debug!(
        terminal_id = %uuid,
        has_input = args.input.is_some(),
        "Writing to terminal"
    );

    let mut terminal = terminal.lock().await;
    let snapshot = match Self::write_and_snapshot(&mut terminal, args.input).await {
      Ok(snapshot) => snapshot,
      Err(response) => return response,
    };

    Self::success(uuid, Some(snapshot))
  }

  async fn handle_snapshot(args: TerminalArgs, terminal_managers: TerminalManagers) -> ToolUseResponse {
    let (uuid, terminal) = match Self::lookup_terminal(args.terminal_id.as_deref(), &terminal_managers).await {
      Ok(result) => result,
      Err(response) => return response,
    };

    let mut terminal = terminal.lock().await;
    let snapshot = match Self::snapshot(&mut terminal, args.timeout.map(Duration::from_secs)).await {
      Ok(snapshot) => snapshot,
      Err(response) => return response,
    };

    Self::success(uuid, Some(snapshot))
  }

  async fn handle_close(args: TerminalArgs, terminal_managers: TerminalManagers) -> ToolUseResponse {
    let uuid = match Self::parse_terminal_id(args.terminal_id.as_deref()) {
      Ok(uuid) => uuid,
      Err(response) => return response,
    };

    let terminal = {
      let mut terminal_managers = terminal_managers.lock().await;
      match terminal_managers.remove(&uuid) {
        Some(terminal) => terminal,
        None => return Self::error("Terminal not found"),
      }
    };

    let close_result = {
      let terminal = terminal.lock().await;
      terminal.close()
    };

    if let Err(error) = close_result {
      terminal_managers.lock().await.insert(uuid, terminal);
      return Self::error(error.to_string());
    }

    Self::success(uuid, None)
  }

  async fn write_and_snapshot(
    terminal: &mut TerminalManager,
    input: Option<String>,
  ) -> ToolResponseResult<TerminalSnapshot> {
    if let Some(input) = normalize_input(input) {
      terminal.write(input).map_err(|error| Self::error(error.to_string()))?;

      Self::snapshot(terminal, Some(SNAPSHOT_TIMEOUT)).await
    } else {
      Self::snapshot(terminal, None).await
    }
  }

  pub async fn get_snapshot(terminal: &mut TerminalManager, timeout: Option<Duration>) -> Result<TerminalSnapshot> {
    match timeout {
      Some(timeout) => match tokio::time::timeout(timeout, terminal.wait_for_snapshot_change()).await {
        Ok(Ok(snapshot)) => Ok(snapshot),
        Ok(Err(error)) => Err(anyhow::anyhow!("Failed to get snapshot: {error}")),
        Err(_) => Ok(terminal.snapshot_text()),
      },
      None => Ok(terminal.snapshot_text()),
    }
  }

  async fn snapshot(terminal: &mut TerminalManager, timeout: Option<Duration>) -> ToolResponseResult<TerminalSnapshot> {
    Self::get_snapshot(terminal, timeout).await.map_err(|error| Self::error(error.to_string()))
  }

  async fn lookup_terminal(
    terminal_id: Option<&str>,
    terminal_managers: &TerminalManagers,
  ) -> ToolResponseResult<(Uuid, SharedTerminal)> {
    let uuid = Self::parse_terminal_id(terminal_id)?;
    let terminal =
      terminal_managers.lock().await.get(&uuid).cloned().ok_or_else(|| Self::error("Terminal not found"))?;

    Ok((uuid, terminal))
  }

  fn parse_terminal_id(terminal_id: Option<&str>) -> ToolResponseResult<Uuid> {
    let Some(terminal_id) = terminal_id else {
      return Err(Self::error("Terminal ID is required"));
    };

    Uuid::from_str(terminal_id).map_err(|_| Self::error("Invalid terminal ID"))
  }

  fn success(terminal_id: Uuid, snapshot: Option<TerminalSnapshot>) -> ToolUseResponse {
    ToolUseResponse::Success(ToolUseResponseSuccess {
      success: true,
      data:    TerminalPayload { terminal_id: terminal_id.to_string(), snapshot }.into(),
      message: None,
    })
  }

  fn error(error: impl Into<String>) -> ToolUseResponse {
    ToolUseResponse::Error(ToolUseResponseError {
      success:         false,
      tool_id:         ToolId::Terminal,
      error:           error.into(),
      subagent_id:     None,
      subagent_status: None,
    })
  }
}

fn normalize_input(input: Option<String>) -> Option<String> {
  input.map(|input| if input.ends_with('\n') { input } else { format!("{input}\n") })
}

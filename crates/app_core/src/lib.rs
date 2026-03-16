#![allow(clippy::redundant_field_names, clippy::field_reassign_with_default)]
#![warn(unused, unused_crate_dependencies)]

pub mod cmd;
pub mod consts;
pub mod engine_manager;
pub mod mcp_runtime;
pub mod menu;
pub mod preview;
pub mod tunnel_handler;
// pub mod payloads;
pub mod setup;

// pub mod debug;

pub mod window_manager;

pub use engine_manager::EngineManager;
use tauri::Wry;
use tauri_specta::Commands;

pub fn commands() -> Commands<Wry> {
  tauri_specta::collect_commands![
    // Control
    cmd::frontend_ready,
    cmd::open_devtools,
    cmd::reload_window,
    cmd::get_build_hash,
    // Memory
    cmd::memory_create,
    cmd::memory_list,
    cmd::memory_read,
    cmd::memory_search,
    cmd::memory_update,
    cmd::memory_delete,
    // MCP
    cmd::mcp_server_list,
    cmd::mcp_server_get,
    cmd::mcp_server_create,
    cmd::mcp_server_update,
    cmd::mcp_server_delete,
    cmd::mcp_server_test_connection,
    cmd::mcp_server_status_list,
    cmd::mcp_server_tools_list,
    // Bun runtime
    cmd::bun_runtime_status,
    cmd::bun_runtime_install_user_local,
    // Providers
    cmd::link_codex_account,
    cmd::unlink_codex_account,
    cmd::link_claude_account,
    cmd::unlink_claude_account,
    // // Session
    cmd::rewind_to,
    cmd::session_stop,
    cmd::session_create,
    cmd::session_delete,
    cmd::session_start,
    cmd::session_history,
    cmd::delete_message,
    cmd::session_get,
    cmd::session_list,
    cmd::session_rename,
    cmd::session_update,
    cmd::answer_question,
    cmd::list_skills,
    cmd::start_plan,
    cmd::complete_plan,
    cmd::continue_plan,
    cmd::cancel_plan,
    cmd::delete_plan,
    cmd::assign_plan_to_session,
    cmd::unassign_plan_from_session,
    cmd::get_terminal_snapshot,
    cmd::close_terminal,
    // LLM
    cmd::send_interrupt,
    cmd::send_prompt,
    cmd::delete_queued_prompt,
    // Provider
    cmd::create_provider,
    cmd::upsert_provider,
    cmd::list_enabled_providers,
    cmd::list_providers,
    cmd::delete_provider,
    cmd::get_models_catalog,
    cmd::create_provider_fnf,
    // Project
    cmd::new_project,
    cmd::edit_project,
    cmd::get_project,
    cmd::list_projects,
    cmd::delete_project,
    cmd::plan_create,
    cmd::plan_list,
    cmd::plan_get,
    cmd::plan_update,
    cmd::plan_cancel,
    cmd::plan_delete,
    // Personality
    cmd::personality_create,
    cmd::personality_update,
    cmd::personality_delete,
    cmd::personality_list,
    // Preview
    cmd::preview_start,
    cmd::preview_stop,
    cmd::preview_reload,
    cmd::preview_status,
    // Slack
    cmd::slack_start_oauth,
    cmd::slack_status,
    cmd::slack_set_enabled,
    cmd::slack_disconnect,
  ]
}

pub fn builder() -> tauri_specta::Builder<tauri::Wry> {
  tauri_specta::Builder::<tauri::Wry>::new().commands(commands())
}

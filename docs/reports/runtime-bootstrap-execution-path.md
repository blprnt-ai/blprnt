# Runtime Bootstrap Execution Path

Issue: `BLP-12`
Date: `2026-03-16`
Owner: Runtime

## Execution Path Map

1. IPC entrypoints land in `crates/app_core/src/cmd/session_commands.rs`.
   - `session_start`, `send_prompt`, `send_interrupt`, `answer_question`, and `rewind_to` all delegate into `EngineManager`.

2. `EngineManager` owns live controller lifecycle in `crates/app_core/src/engine_manager.rs`.
   - `session_start` checks the live controller registry, otherwise calls `init_engine`.
   - `init_engine` rebuilds sandbox roots, constructs `ControllerConfig`, creates `Controller`, starts the runtime, and stores the controller in `controllers`.
   - `send_prompt`, `send_interrupt`, `answer_question_with_idempotency`, `close_terminal`, and `get_terminal_snapshot` all route through the controller registry.

3. `Controller` bridges prompts and runtime in `crates/engine_v2/src/controller.rs`.
   - `push_prompt` parses the prompt into a `QueueItem`, emits `PromptQueued`, pushes into the queue, and starts the runtime if it is not already running.
   - `run` spawns `Runtime::inner_run`.
   - `stop` cancels the shared token and resets it for future turns.
   - `answer_question_with_idempotency` forwards directly to runtime ask-question claim handling.

4. `Runtime` drives turns and steps in `crates/engine_v2/src/runtime/mod.rs`.
   - `inner_run` drains the queue and calls `next_turn`.
   - `next_turn` builds `RuntimeContext`, appends the user prompt to session history, runs pre-turn hooks, then enters `run_loop`.
   - `run_loop` builds the provider request, subscribes to `ProviderDispatch`, processes streamed events, executes tool futures, persists tool responses, and runs post-step hooks.
   - Turn-level cancellation is handled with `tokio::select!` on the runtime cancel token.

5. `RuntimeContext` resolves provider + tool schema in `crates/engine_v2/src/runtime/context.rs`.
   - Reads session/project state, selects the model provider, builds the tool schema, attaches MCP tools, and assembles the final `ChatRequest`.

6. Provider streaming fans back into runtime through `crates/providers/src/provider_adapter.rs` and `crates/engine_v2/src/runtime/provider_event_handler.rs`.
   - `ProviderAdapter::stream_conversation` delegates to the concrete provider implementation.
   - `ProviderEventHandler::process_provider_events` persists assistant reasoning, assistant responses, token usage, provider errors, and tool calls.

7. Tool dispatch splits in `crates/engine_v2/src/runtime/tool_call_handler.rs`.
   - `AskQuestion` uses `crates/engine_v2/src/runtime/ask_question_handler.rs`.
   - `SubAgent` uses `crates/engine_v2/src/runtime/subagent_handler.rs`.
   - All other runtime-facing tools use `crates/engine_v2/src/runtime/basic_tool_handler.rs`, which builds `ToolUseContext` and calls `crates/tools/src/tools.rs`.

8. Ask-question claims close the loop through runtime state in `crates/engine_v2/src/runtime/mod.rs`.
   - Pending question senders live in `user_interaction_requests`.
   - `claim_ask_question_interaction` enforces single-winner semantics and Slack idempotency normalization.
   - `maybe_handle_user_interaction_requests` resumes unanswered questions found in persistence when the runtime restarts.

## Highest-Risk Findings

1. Assistant response messages were being parented with `turn_id` instead of `step_id` in `crates/engine_v2/src/runtime/provider_event_handler.rs`.
   - Impact: response threading is lost for normal assistant output, which can mis-shape conversation trees and downstream consumers that depend on parent linkage.
   - Side effect: top-level response filtering in Slack-facing code is more likely to misclassify nested responses.
   - Status: fixed in this heartbeat.

2. `delete_session` removed live controllers without stopping them in `crates/app_core/src/engine_manager.rs`.
   - Impact: a deleted session could leave an orphaned runtime task running against torn-down session state.
   - Status: fixed in this heartbeat.

3. `session_update` could panic on non-running sessions via `controllers.get(...).unwrap()` in `crates/app_core/src/engine_manager.rs`.
   - Impact: updating persisted session settings before `session_start` could crash the command path instead of behaving as an offline config update.
   - Status: fixed in this heartbeat by making live queue sync conditional on an active controller.

4. Provider dispatch still depends on a 120-second idle timeout in `crates/engine_v2/src/runtime/mod.rs`.
   - Impact: swallowed provider failures or missing terminal events degrade into slow turn failure instead of explicit typed errors.
   - Status: not changed; this remains a follow-up hardening target.

5. Subagent completion depends on broadcast observation of `TurnStop` in `crates/engine_v2/src/runtime/subagent_handler.rs`.
   - Impact: if event delivery or message ordering shifts, subagent result capture can fall back to timeout behavior.
   - Status: unchanged; needs a dedicated completion contract review.

## Recommended Next Fixes

1. Add runtime-level regression coverage for provider event persistence, especially response/reasoning parent linkage and token usage attachment.
2. Replace the idle-timeout fallback with explicit provider task completion and surfaced provider termination errors.
3. Audit subagent completion against cancellation, lagged broadcasts, and partial-response delivery.

## Verification

- Run `cargo test -p engine_v2`
- Run `cargo test -p app_core`
- Spot-check response/message parent linkage in a live session after one streamed assistant turn

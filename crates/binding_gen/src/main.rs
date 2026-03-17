use common::blprnt::BlprntEventKind;
use common::blprnt::TunnelMessage;
use common::blprnt_dispatch::SessionEvent;
use common::shared::prelude::DeleteQueuedPromptOutcome;
use common::shared::prelude::DeleteQueuedPromptRequest;
use common::tools::question::AskQuestionArgs;
use common::tools::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  println!("Generating bindings...");

  tauri_specta::Builder::<tauri::Wry>::new()
    .commands(app_core::commands())
    // File tools
    .typ::<FilesReadArgs>()
    .typ::<ApplyPatchArgs>()
    .typ::<RgSearchArgs>()
    // Memory tools
    .typ::<MemoryWriteArgs>()
    .typ::<MemorySearchArgs>()
    // Shell tools
    .typ::<ShellArgs>()
    .typ::<TerminalArgs>()
    .typ::<AskQuestionArgs>()
    // Primer tools
    .typ::<GetPrimerArgs>()
    .typ::<UpdatePrimerArgs>()
    // Subagent tools
    .typ::<SubAgentArgs>()
    // Plan tools
    .typ::<PlanCreateArgs>()
    .typ::<PlanListArgs>()
    .typ::<PlanGetArgs>()
    .typ::<PlanUpdateArgs>()
    .typ::<PlanDeleteArgs>()
    .typ::<DeleteQueuedPromptRequest>()
    .typ::<DeleteQueuedPromptOutcome>()

    .typ::<BlprntEventKind>()
    .typ::<SessionEvent>()
    .typ::<TunnelMessage>()

    .export(
      specta_typescript::Typescript::default()
        .bigint(specta_typescript::BigIntExportBehavior::Number)
        .formatter(specta_typescript::formatter::prettier),
      "./src/bindings.ts"
    )
    .expect("failed to export bindings");

  println!("Bindings generated successfully");

  Ok(())
}

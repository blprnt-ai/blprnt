# Frontend Shell Map

Date: 2026-03-16
Owner: Senior Product Engineer
Source task: BLP-11

## Scope

This map covers the React desktop shell in `src/`, focusing on boot, MobX model boundaries, the Tauri event bridge, Dockview composition, and the session/project/plan/provider workflows that are highest leverage for product stability.

## Runtime Spine

```text
main.tsx
  -> AppViewModel.init()
    -> frontendReady()
    -> Tauri event listeners
    -> AppModel / ProviderModel refresh
  -> DockviewProvider
    -> SidebarViewmodel + DockviewLayoutViewModel
    -> Project / Session / Plan / Settings panels
```

## Boot And Event Flow

- `src/main.tsx`
  - Creates the app root, waits on the backend `backendReady` event, applies the stored theme, and mounts shared providers.
- `src/app.tsx`
  - Gates the shell on `AppViewModel.isLoading`, then waits on the backend-ready promise before mounting Dockview.
- `src/app.viewmodel.tsx`
  - Calls `AppModel.frontendReady()`, starts the Tauri event bridge, installs window listeners, and refreshes provider state for the shell header/settings flows.
- `src/lib/api/tauri/command.api.ts`
  - Wraps the frontend-facing Tauri control commands.
- `crates/app_core/src/cmd/control_commands.rs`
  - `frontend_ready` shows the main window, tears down the loader window, installs the native menu, and emits `backendReady`.
- `src/lib/events/lib.ts`
  - Thin typed wrapper over Tauri `listen` and `once`.
- `src/lib/events/listener.ts`
  - Converts raw Tauri events into the app-local `globalEventBus` event types.
- `src/lib/events/event-bus.ts`
  - In-memory fan-out for session, internal, oauth, backend-ready, and settings-adjacent events.

## Model Boundaries

- `src/lib/models/app.model.ts`
  - Global shell state for loading, focus, model catalog, personalities, and skills.
- `src/lib/models/project.model.ts`
  - Registry-backed project cache; refreshes itself from internal project events.
- `src/lib/models/session.model.ts`
  - Registry-backed session cache; owns runtime status, token usage, plan assignment, and subscriptions to session control/llm events.
- `src/lib/models/plan.model.ts`
  - Registry-backed plan cache keyed by `projectId:planId`; refreshes on internal `plan_updated`.
- `src/lib/models/provider.model.ts`
  - Thin CRUD wrapper for provider credentials and subscription-link commands.

## Panel Shell

- `src/context/dockview-context.tsx`
  - Builds the main two-column shell: sidebar on the left, Dockview workspace on the right.
- `src/components/dockview/dockview-layout.viewmodel.tsx`
  - Owns Dockview API references, layout persistence, open/close behavior, and the session-stop side effect on session panel close.
- `src/components/dockview/use-dockview.ts`
  - Hooks Dockview events, persists layout changes, tracks the active panel, and binds `Cmd/Ctrl+T` and `Cmd/Ctrl+W`.
- `src/components/dockview/content-components.tsx`
  - Maps Dockview content ids to panel implementations.
- `src/components/organisms/sidebar/sidebar.viewmodel.tsx`
  - Mirrors open panels into sidebar state, loads projects and sessions, and opens project/session/preview/settings panels.

## Workflow Trace

### Session Workflow

- Creation starts in `src/components/dialogs/session/new-session-dialog.tsx`.
- `SessionModel.create()` in `src/lib/models/session.model.ts` calls the Tauri create command, then emits an internal `session_added`.
- Sidebar consumers react in `src/components/organisms/sidebar/sidebar.viewmodel.tsx` and session trees in `src/components/organisms/trees/sessions-tree.viewmodel.tsx`.
- Opening a session panel routes through `src/components/organisms/sidebar/sidebar.viewmodel.tsx`, `src/components/dockview/content-components.tsx`, and `src/components/panels/session/session-panel.provider.tsx`.
- The conversation surface is driven by `src/components/panels/session/session-panel.viewmodel.tsx`, which loads history, listens to session bus events, manages prompt queue state, and syncs plan assignment.

### Project Workflow

- Projects open through sidebar actions in `src/components/organisms/sidebar/sidebar.viewmodel.tsx`.
- The project panel in `src/components/panels/project-panel.tsx` mounts a `ProjectEditorViewModel`.
- `src/components/organisms/project/project-editor.viewmodel.tsx` handles autosave, working-directory validation, and create/update/delete.
- Project CRUD calls fan through `src/lib/api/tauri/project.api.ts` and reflect back into `ProjectModel` via internal events.

### Plan Workflow

- Project plan listing lives in `src/components/organisms/project/project-plans-list-v2.tsx` and `src/components/organisms/project/plan-list-viewmodel.tsx`.
- Plan panels open through Dockview and mount `src/components/panels/plan/plan-panel.provider.tsx`.
- `src/components/panels/plan/plan-panel.viewmodel.tsx` keeps a draft copy, autosaves content/todos, and listens for runtime-generated `plan_update` tool results.
- Session plan assignment is orchestrated in `src/components/panels/session/session-panel.viewmodel.tsx` through `PlanModel.assignToSession()` and `PlanModel.unassignFromSession()`.

### Provider Workflow

- Settings tabs are defined in `src/components/views/settings/settings-page.tsx`.
- Provider CRUD and subscription linking live in `src/components/views/settings/components/providers-page/providers-page-viewmodel.tsx`.
- UI entry points are `custom-providers.tsx` and `fnf-providers.tsx`, backed by `src/lib/api/tauri/providers.api.ts`.
- App-wide "linked account" affordances depend on `AppViewModel.refreshProviders()` in `src/app.viewmodel.tsx`.

## Prioritized Risks

### 1. Shared session state can go stale after a single session panel closes

Evidence:

- `SessionModel.getOrCreate()` only calls `init()` for brand-new instances in `src/lib/models/session.model.ts`.
- `SessionPanelViewmodel.destroy()` calls `this.session?.destroy()` in `src/components/panels/session/session-panel.viewmodel.tsx`.

Why this is risky:

- `SessionModel` is a registry singleton, so one panel unmount tears down event subscriptions for every future consumer of that same session id.
- Re-opening the session reuses the same cached instance without re-subscribing, so running state, token usage, and model override updates can silently stop.

### 2. "Read" access to a session starts the engine runtime as a side effect

Evidence:

- `SessionModel.get()` calls `tauriSessionApi.start()` in `src/lib/models/session.model.ts`.
- That method is used by sidebar hydration, the session panel, and the edit dialog in:
  - `src/components/organisms/sidebar/sidebar.viewmodel.tsx`
  - `src/components/panels/session/session-panel.viewmodel.tsx`
  - `src/components/dialogs/session/edit-session-dialog.tsx`

Why this is risky:

- Viewing a newly created session in the sidebar or opening the edit dialog can start backend session machinery unexpectedly.
- This makes session lifecycle hard to reason about and increases the chance of "why did this session wake up?" product defects.

### 3. Sidebar close path bypasses the runtime-safe session shutdown path

Evidence:

- `DockviewLayoutViewModel.closePanel()` stops the session before closing the tab in `src/components/dockview/dockview-layout.viewmodel.tsx`.
- Sidebar close actions call a different `SidebarViewmodel.closePanel()` that directly closes the Dockview panel in `src/components/organisms/sidebar/sidebar.viewmodel.tsx`.
- The tree UI uses that bypass from `src/components/organisms/trees/session-tree.tsx` and `src/components/organisms/trees/subagents-tree.tsx`.

Why this is risky:

- The same user-visible action behaves differently depending on where the tab is closed.
- A running session can keep executing after the sidebar says its tab is gone.

### 4. Project autosave can silently degrade after save failures

Evidence:

- `ProjectEditorViewModel.save()` destroys all reactions before `update()`, then swallows errors in `src/components/organisms/project/project-editor.viewmodel.tsx`.
- `update()` simply returns when the form is invalid, leaving dirty local state behind.

Why this is risky:

- A failed autosave can leave the editor without live reactions or with stale dirty state.
- The user gets no reliable signal that later edits are no longer being persisted.

### 5. Provider settings use an implicit singleton viewmodel

Evidence:

- `SettingsPage` renders `ProvidersPage` directly in `src/components/views/settings/settings-page.tsx`.
- `ProvidersPageProvider` exists but is unused in `src/components/views/settings/components/providers-page/providers-page-provider.tsx`.
- `ProvidersPageViewmodelContext` is created with `new ProvidersPageViewmodel()` as its default value in `src/components/views/settings/components/providers-page/providers-page-viewmodel.tsx`.

Why this is risky:

- Multiple settings surfaces can share one hidden mutable provider viewmodel instance.
- Unsaved input and loading flags can bleed across tabs or remounts, which is the wrong default for credentials UI.

## Recommended First Hardening Tasks

### 1. Make session registry ownership explicit

- Keep session event subscriptions owned by the registry, not by panel teardown.
- Separate panel cleanup from shared model cleanup.
- Add a true read-only `SessionModel.fetch()` path that does not call `sessionStart`.

Verification:

- Open a session, close the panel, reopen it, and confirm status/token events still update.
- Open the edit dialog for an idle session and confirm no backend start command is issued.

### 2. Unify all session tab close flows through one runtime-safe path

- Route sidebar/tree close actions through `DockviewLayoutViewModel.closePanel()`.
- Keep stop semantics in one place.

Verification:

- Start a session, close it from the sidebar tree, and confirm the backend session stops.
- Repeat via tab close button and hotkey to confirm identical behavior.

### 3. Stabilize project and provider editors

- Refactor project autosave so failed writes do not tear down reactions.
- Surface save failures in the project panel.
- Instantiate the providers page viewmodel per page mount through the existing provider wrapper.

Verification:

- Enter an invalid working directory, then fix it, and confirm autosave still resumes.
- Open two settings panels and confirm provider form state does not leak between them.

## Verification Baseline

- `pnpm build`

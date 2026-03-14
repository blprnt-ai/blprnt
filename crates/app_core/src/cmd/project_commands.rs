use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use common::errors::AppCoreError;
use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use common::errors::ToolError;
use common::plan_utils::PlanFrontmatter;
use common::plan_utils::apply_plan_content_patch;
use common::plan_utils::build_plan_id;
use common::plan_utils::build_write_context;
use common::plan_utils::ensure_plan_dir;
use common::plan_utils::get_plan_content;
use common::plan_utils::is_plan_file;
use common::plan_utils::parse_frontmatter;
use common::plan_utils::render_plan_content;
use common::plan_utils::resolve_plan_directory;
use common::tools::PlanCreateArgs;
use common::tools::PlanDocumentStatus;
use common::tools::PlanGetPayload;
use common::tools::PlanListQuery;
use common::tools::PlanListSortBy;
use common::tools::PlanUpdateArgs;
use common::tools::ProjectPlanListItem;
use common::tools::ProjectPlanListPayload;
use common::tools::SortDirection;
use persistence::prelude::ProjectModelV2;
use persistence::prelude::ProjectPatchV2;
use persistence::prelude::ProjectRecord;
use tauri::State;

use crate::engine_manager::EngineManager;

#[tauri::command]
#[specta::specta]
pub async fn new_project(
  manager: State<'_, Arc<EngineManager>>,
  name: String,
  working_directories: Vec<PathBuf>,
  agent_primer: Option<String>,
) -> TauriResult<ProjectRecord> {
  tracing::debug!("New Project: {:?}", name);

  let project_model = ProjectModelV2::new(name, working_directories.into(), agent_primer);

  manager.create_project(project_model).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn edit_project(
  manager: State<'_, Arc<EngineManager>>,
  id: String,
  project: ProjectPatchV2,
) -> TauriResult<ProjectRecord> {
  tracing::debug!("Edit Project: {:?}", id);
  let id = id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.edit_project(id, project).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn get_project(manager: State<'_, Arc<EngineManager>>, project_id: String) -> TauriResult<ProjectRecord> {
  tracing::debug!("Get Project: {:?}", project_id);
  let project_id = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.get_project(project_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn list_projects(manager: State<'_, Arc<EngineManager>>) -> TauriResult<Vec<ProjectRecord>> {
  tracing::debug!("List Projects");
  manager.list_projects().await.map(|projects| projects.into_iter().collect()).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn delete_project(manager: State<'_, Arc<EngineManager>>, project_id: String) -> TauriResult<()> {
  tracing::debug!("Delete Project: {:?}", project_id);
  let project_id = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.delete_project(project_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn plan_list(project_id: String, query: Option<PlanListQuery>) -> TauriResult<ProjectPlanListPayload> {
  tracing::debug!("List Plans for project: {:?}", project_id);
  let project_id: persistence::prelude::SurrealId = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  let query = query.unwrap_or_default();

  let mut items = list_project_plan_items(project_id).into_tauri()?;

  if let Some(search) = query.search {
    let search = search.to_lowercase();
    items.retain(|item| {
      item.id.to_lowercase().contains(&search)
        || item.name.to_lowercase().contains(&search)
        || item.description.to_lowercase().contains(&search)
    });
  }

  if let Some(status_filter) = query.status_filter {
    items.retain(|item| status_filter.contains(&item.status));
  }

  let sort = query.sort.unwrap_or_default();
  items.sort_by(|left, right| {
    let ordering = match sort.by {
      PlanListSortBy::Name => left.name.cmp(&right.name),
      PlanListSortBy::CreatedAt => left.created_at.cmp(&right.created_at),
      PlanListSortBy::UpdatedAt => left.updated_at.cmp(&right.updated_at),
    };

    match sort.direction {
      SortDirection::Asc => ordering,
      SortDirection::Desc => ordering.reverse(),
    }
  });

  Ok(ProjectPlanListPayload { items })
}

#[tauri::command]
#[specta::specta]
pub async fn plan_get(
  manager: State<'_, Arc<EngineManager>>,
  project_id: String,
  plan_id: String,
) -> TauriResult<PlanGetPayload> {
  tracing::debug!("Get Plan: {:?}", plan_id);
  let project_id: persistence::prelude::SurrealId = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;

  manager.plan_get(project_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn plan_create(
  _manager: State<'_, Arc<EngineManager>>,
  project_id: String,
  args: PlanCreateArgs,
) -> TauriResult<PlanGetPayload> {
  tracing::debug!("Create Plan");
  let project_id: persistence::prelude::SurrealId = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;

  create_plan(project_id, args).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn plan_update(
  _manager: State<'_, Arc<EngineManager>>,
  project_id: String,
  args: PlanUpdateArgs,
) -> TauriResult<PlanGetPayload> {
  tracing::debug!("Update Plan: {:?}", args.id);
  let project_id: persistence::prelude::SurrealId = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;

  let plan = get_plan_content(project_id.clone(), args.id.clone()).into_tauri()?;
  ensure_plan_editable(&args, &plan.status).map_err(anyhow::Error::from).into_tauri()?;

  update_plan(project_id, args).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn plan_cancel(
  manager: State<'_, Arc<EngineManager>>,
  project_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Cancel Plan: {:?}", plan_id);
  let project_id: persistence::prelude::SurrealId = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;

  manager.cancel_plan_for_project(project_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn plan_delete(
  manager: State<'_, Arc<EngineManager>>,
  project_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Delete Plan: {:?}", plan_id);
  let project_id: persistence::prelude::SurrealId = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;

  manager.delete_plan_for_project(project_id, plan_id).await.into_tauri()
}

fn create_plan(project_id: persistence::prelude::SurrealId, args: PlanCreateArgs) -> anyhow::Result<PlanGetPayload> {
  let plan_directory = resolve_plan_directory(project_id.clone())?;
  let base_path = PathBuf::from(&plan_directory.path);
  ensure_plan_dir(&base_path)?;

  let now = Utc::now().to_rfc3339();
  let plan_id = build_plan_id(&args.name);
  let write_context = build_write_context(plan_id, &base_path, now.clone(), now);
  let path = PathBuf::from(&write_context.plan_path);

  if path.exists() {
    return Err(
      common::errors::ToolError::FileWriteFailed {
        path:  write_context.plan_path,
        error: "plan already exists".to_string(),
      }
      .into(),
    );
  }

  let frontmatter = PlanFrontmatter {
    name:              args.name,
    description:       args.description,
    todos:             args.todos.unwrap_or_default(),
    created_at:        write_context.created_at,
    updated_at:        write_context.updated_at,
    status:            PlanDocumentStatus::Pending,
    parent_session_id: None,
  };

  let content = render_plan_content(&frontmatter, &args.content)?;
  std::fs::write(&path, content).map_err(|e| common::errors::ToolError::FileWriteFailed {
    path:  path.display().to_string(),
    error: e.to_string(),
  })?;

  get_plan_content(project_id, write_context.plan_id)
}

fn ensure_plan_editable(args: &PlanUpdateArgs, plan_status: &PlanDocumentStatus) -> Result<(), AppCoreError> {
  if args.status.is_some()
    && args.name.is_none()
    && args.description.is_none()
    && args.content.is_none()
    && args.content_patch.is_none()
    && args.todos.is_none()
  {
    return Ok(());
  }

  if *plan_status != PlanDocumentStatus::Pending {
    let status = match plan_status {
      PlanDocumentStatus::Completed => "Complete".to_string(),
      _ => format!("{:?}", plan_status),
    };

    return Err(AppCoreError::PlanNotPending { plan_id: args.id.clone(), status });
  }

  Ok(())
}

fn update_plan(project_id: persistence::prelude::SurrealId, args: PlanUpdateArgs) -> anyhow::Result<PlanGetPayload> {
  if args.content.is_some() && args.content_patch.is_some() {
    return Err(ToolError::General("plan_update content and content_patch are mutually exclusive".to_string()).into());
  }

  let plan_directory = resolve_plan_directory(project_id.clone())?;
  let base_path = PathBuf::from(&plan_directory.path);
  let path = base_path.join(&args.id);

  let content = std::fs::read_to_string(&path).map_err(|e| common::errors::ToolError::FileReadFailed {
    path:  path.display().to_string(),
    error: e.to_string(),
  })?;
  let (frontmatter, body) = parse_frontmatter(&content)?;
  let mut meta = frontmatter.into_meta();

  if let Some(name) = args.name {
    meta.name = name;
  }
  if let Some(description) = args.description {
    meta.description = description;
  }
  if let Some(todos) = args.todos {
    meta.todos = todos;
  }
  if let Some(status) = args.status {
    meta.status = status;
  }

  let updated_body = if let Some(content) = args.content {
    content
  } else if let Some(content_patch) = args.content_patch {
    apply_plan_content_patch(&body, &content_patch)?
  } else {
    body
  };
  meta.updated_at = Utc::now().to_rfc3339();

  let frontmatter = PlanFrontmatter {
    name:              meta.name,
    description:       meta.description,
    todos:             meta.todos,
    created_at:        meta.created_at,
    updated_at:        meta.updated_at,
    status:            meta.status,
    parent_session_id: meta.parent_session_id,
  };

  let updated_content = render_plan_content(&frontmatter, &updated_body)?;
  std::fs::write(&path, updated_content).map_err(|e| common::errors::ToolError::FileWriteFailed {
    path:  path.display().to_string(),
    error: e.to_string(),
  })?;

  get_plan_content(project_id, args.id)
}

fn list_project_plan_items(project_id: persistence::prelude::SurrealId) -> anyhow::Result<Vec<ProjectPlanListItem>> {
  let plan_directory = resolve_plan_directory(project_id)?;
  let base_path = PathBuf::from(&plan_directory.path);
  ensure_plan_dir(&base_path)?;

  let mut items = Vec::new();
  let entries = std::fs::read_dir(&base_path)?;

  for entry in entries {
    let entry = entry?;
    let path = entry.path();
    if !path.is_file() || !is_plan_file(&path) {
      continue;
    }

    let content = match std::fs::read_to_string(&path) {
      Ok(content) => content,
      Err(_) => continue,
    };

    let (frontmatter, _) = match parse_frontmatter(&content) {
      Ok(result) => result,
      Err(_) => continue,
    };

    let meta = frontmatter.into_meta();
    let status = meta.status.clone();
    let plan_id = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let plan_payload = PlanGetPayload {
      id: plan_id.clone(),
      name: meta.name.clone(),
      description: meta.description.clone(),
      content: String::new(),
      created_at: meta.created_at.clone(),
      updated_at: meta.updated_at.clone(),
      status,
      parent_session_id: meta.parent_session_id.clone(),
      todos: meta.todos.clone(),
    };
    items.push(ProjectPlanListItem {
      id:                plan_id.clone(),
      name:              meta.name,
      description:       meta.description,
      created_at:        meta.created_at,
      updated_at:        meta.updated_at,
      status:            plan_payload.status,
      parent_session_id: plan_payload.parent_session_id,
    });
  }

  Ok(items)
}

#[cfg(test)]
mod tests {
  use common::errors::AppCoreError;
  use common::tools::PlanContentPatch;
  use common::tools::PlanDocumentStatus;
  use common::tools::PlanUpdateArgs;

  use super::ensure_plan_editable;
  use super::update_plan;

  fn sample_patch() -> PlanContentPatch {
    PlanContentPatch {
      hunks: vec![common::tools::PlanContentPatchHunk {
        before: vec!["alpha".to_string()],
        delete: vec!["beta".to_string()],
        insert: vec!["delta".to_string()],
        after:  vec!["gamma".to_string()],
      }],
    }
  }

  #[test]
  fn ensure_plan_editable_allows_when_pending() {
    let args = PlanUpdateArgs {
      id:            "plan-1".to_string(),
      name:          None,
      description:   None,
      content:       None,
      content_patch: None,
      todos:         None,
      status:        None,
    };
    let result = ensure_plan_editable(&args, &PlanDocumentStatus::Pending);

    assert!(result.is_ok());
  }

  #[test]
  fn ensure_plan_editable_rejects_when_non_pending() {
    let args = PlanUpdateArgs {
      id:            "plan-1".to_string(),
      name:          Some("Name".to_string()),
      description:   None,
      content:       None,
      content_patch: None,
      todos:         None,
      status:        None,
    };
    let result = ensure_plan_editable(&args, &PlanDocumentStatus::Completed);

    assert!(
      matches!(result, Err(AppCoreError::PlanNotPending { plan_id, status }) if plan_id == "plan-1" && status == "Complete")
    );
  }

  #[test]
  fn ensure_plan_editable_allows_status_only_update() {
    let args = PlanUpdateArgs {
      id:            "plan-1".to_string(),
      name:          None,
      description:   None,
      content:       None,
      content_patch: None,
      todos:         None,
      status:        Some(PlanDocumentStatus::Completed),
    };
    let result = ensure_plan_editable(&args, &PlanDocumentStatus::Completed);

    assert!(result.is_ok());
  }

  #[test]
  fn ensure_plan_editable_rejects_content_patch_when_non_pending() {
    let args = PlanUpdateArgs {
      id:            "plan-1".to_string(),
      name:          None,
      description:   None,
      content:       None,
      content_patch: Some(Default::default()),
      todos:         None,
      status:        None,
    };
    let result = ensure_plan_editable(&args, &PlanDocumentStatus::Completed);

    assert!(
      matches!(result, Err(AppCoreError::PlanNotPending { plan_id, status }) if plan_id == "plan-1" && status == "Complete")
    );
  }

  #[test]
  fn update_plan_rejects_content_and_content_patch_together() {
    let args = PlanUpdateArgs {
      id:            "plan-1".to_string(),
      name:          None,
      description:   None,
      content:       Some("replacement body".to_string()),
      content_patch: Some(sample_patch()),
      todos:         None,
      status:        None,
    };

    let error = update_plan(persistence::prelude::SurrealId::default(), args).unwrap_err().to_string();

    assert!(error.contains("plan_update content and content_patch are mutually exclusive"));
  }
}

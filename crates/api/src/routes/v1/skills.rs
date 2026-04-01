use std::collections::HashSet;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::routing::get;
use ts_rs::TS;

use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

#[derive(Debug, Clone, serde::Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct Skill {
  name:        String,
  description: String,
  path:        String,
  source:      String,
}

impl From<skills::SkillMetadata> for Skill {
  fn from(skill: skills::SkillMetadata) -> Self {
    let source = match skill.source {
      skills::SkillSource::User => "user",
      skills::SkillSource::Builtin => "builtin",
    };

    Self {
      name:        skill.name,
      description: skill.description,
      path:        skill.path.to_string_lossy().to_string(),
      source:      source.to_string(),
    }
  }
}

pub fn routes() -> Router {
  Router::new().route("/skills", get(list_skills))
}

#[utoipa::path(
  get,
  path = "/skills",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List available skills", body = [Skill]),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "skills"
)]
pub(super) async fn list_skills(Extension(extension): Extension<RequestExtension>) -> ApiResult<Json<Vec<Skill>>> {
  let assigned_skill_names: HashSet<&str> = extension
    .employee
    .runtime_config
    .as_ref()
    .and_then(|config| config.skill_stack.as_ref())
    .into_iter()
    .flatten()
    .map(|skill| skill.name.as_str())
    .collect();

  let skills = skills::list_skills()
    .map_err(|err| ApiErrorKind::InternalServerError(err.to_string().into()))?
    .into_iter()
    .filter(|skill| !assigned_skill_names.contains(skill.name.as_str()))
    .map(Skill::from)
    .collect();
  Ok(Json(skills))
}

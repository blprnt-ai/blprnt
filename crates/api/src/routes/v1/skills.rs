use axum::Json;
use axum::Router;
use axum::routing::get;
use ts_rs::TS;

use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;

#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export)]
struct Skill {
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

async fn list_skills() -> ApiResult<Json<Vec<Skill>>> {
  let skills = skills::list_skills()
    .map_err(|err| ApiErrorKind::InternalServerError(err.to_string().into()))?
    .into_iter()
    .map(Skill::from)
    .collect();
  Ok(Json(skills))
}

use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct Project {
  pub name:        String,
  pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProjectRecord {
  pub id:          RecordId,
  pub name:        String,
  pub description: String,
  pub sessions:    Vec<SessionRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct Session {
  pub name:        String,
  pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct SessionRecord {
  pub id:          RecordId,
  pub name:        String,
  pub description: String,
  pub project:     RecordId,
}

#[derive(Debug, SurrealValue)]
pub struct Record {
  pub id: RecordId,
}

#[cfg(test)]
mod tests {
  use surrealdb::Surreal;
  use surrealdb::engine::remote::ws::Ws;
  use surrealdb_types::RecordId;

  use super::*;

  #[tokio::test]
  async fn test_serialize() {
    let db = Surreal::new::<Ws>("localhost:8000").await.unwrap();
    db.use_ns("app").use_db("main").await.unwrap();

    println!("Connected to database");

    let _ = db
      .query(
        r#"
        DELETE FROM project;
        REMOVE TABLE project;
        DELETE FROM session;
        REMOVE TABLE session;

        DEFINE TABLE project SCHEMALESS;
        DEFINE TABLE session SCHEMALESS;

        DEFINE FIELD project ON session TYPE option<record<project>> REFERENCE;
        DEFINE FIELD sessions ON project COMPUTED <~session;
      "#,
      )
      .await
      .unwrap();

    let project = Project { name: "Test".to_string(), description: "Test".to_string() };
    let project_id = RecordId::new("project", "one");
    let _project: Record = db.create(project_id.clone()).content(project).await.unwrap().unwrap();

    let session = Session { name: "Test".to_string(), description: "Test".to_string() };
    let session_id = RecordId::new("sessions", "one");
    let _: Record = db.create(session_id.clone()).content(session).await.unwrap().unwrap();
    let session: Option<Record> = db
      .query("UPDATE $record_id SET project = $project_id")
      .bind(("record_id", session_id.clone()))
      .bind(("project_id", project_id.clone()))
      .await
      .unwrap()
      .take(0)
      .unwrap();

    println!("Session: {:?}", session);

    let mut result = db.query("SELECT *, sessions.* FROM project").await.unwrap();

    println!("Result: {:#?}", result);

    let project: Vec<ProjectRecord> = result.take(0).unwrap();

    println!("Project: {:#?}", project);
    println!("Session: {:#?}", session);
  }
}

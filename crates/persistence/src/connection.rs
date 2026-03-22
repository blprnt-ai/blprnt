use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use anyhow::Result;
use lazy_static::lazy_static;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use tokio::sync::OnceCell;

use crate::models::EmployeeModel;
use crate::models::IssueModel;
use crate::models::RunModel;
use crate::models::TurnModel;
use crate::prelude::ProjectModel;
use crate::prelude::ProviderModel;

pub type DbConnection = Surreal<Db>;

lazy_static! {
  static ref DB: OnceCell<DbConnection> = OnceCell::new();
}

static MIGRATED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct SurrealConnection;

impl SurrealConnection {
  pub async fn db() -> DbConnection {
    let db = Self::connect().await;

    if !MIGRATED.load(Ordering::Relaxed) {
      MIGRATED.store(true, Ordering::Relaxed);
      Self::migrate(db.clone()).await.expect("Failed to migrate database");
    }

    db.clone()
  }

  #[cfg(not(feature = "testing"))]
  async fn connect() -> DbConnection {
    DB.get_or_init(|| async {
      use shared::paths;
      use surrealdb::engine::local::RocksDb;
      let path = paths::blprnt_home().join("data");

      tracing::info!("Connecting to surrealdb");
      let db = Surreal::new::<RocksDb>(path).await.expect("Failed to connect to surrealdb");
      tracing::info!("Connected to surrealdb");

      db.use_ns("app").use_db("main").await.expect("Failed to use namespace and database");

      db
    })
    .await
    .clone()
  }

  #[cfg(feature = "testing")]
  async fn connect() -> DbConnection {
    DB.get_or_init(|| async {
      use surrealdb::engine::local::Mem;

      let db = Surreal::new::<Mem>(()).await.expect("Failed to create in-memory surrealdb");
      db.use_ns("app").use_db("main").await.expect("Failed to use namespace and database");
      db
    })
    .await
    .clone()
  }

  async fn migrate(db: DbConnection) -> Result<()> {
    let _ = ProviderModel::migrate(&db).await;
    let _ = ProjectModel::migrate(&db).await;
    let _ = EmployeeModel::migrate(&db).await;
    let _ = RunModel::migrate(&db).await;
    let _ = TurnModel::migrate(&db).await;
    let _ = IssueModel::migrate(&db).await;

    Ok(())
  }
}

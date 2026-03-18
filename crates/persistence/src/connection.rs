use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use anyhow::Result;
#[cfg(not(feature = "testing"))]
use lazy_static::lazy_static;
use surrealdb::Surreal;
#[cfg(feature = "testing")]
use surrealdb::engine::local::Db;
#[cfg(feature = "testing")]
use surrealdb::engine::local::Mem;
#[cfg(not(feature = "testing"))]
use surrealdb::engine::remote::ws::Client;
#[cfg(not(feature = "testing"))]
use surrealdb::engine::remote::ws::Ws;
use tokio::sync::OnceCell;

use crate::prelude::ProjectModelV2;
use crate::prelude::ProviderModelV2;

const SURREAL_DB_PORT: u16 = 14145;

#[cfg(not(feature = "testing"))]
pub type DbConnection = Surreal<Client>;
#[cfg(feature = "testing")]
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
      Self::migrate(db.clone()).await.expect("Failed to migrate database");
      MIGRATED.store(true, Ordering::Relaxed);
    }

    db.clone()
  }

  #[cfg(not(feature = "testing"))]
  async fn connect() -> DbConnection {
    DB.get_or_init(|| async {
      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      tracing::info!("Connecting to surrealdb");
      let db =
        Surreal::new::<Ws>(format!("127.0.0.1:{}", SURREAL_DB_PORT)).await.expect("Failed to connect to surrealdb");
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
      let db = Surreal::new::<Mem>(()).await.expect("Failed to create in-memory surrealdb");
      db.use_ns("app").use_db("main").await.expect("Failed to use namespace and database");
      db
    })
    .await
    .clone()
  }

  async fn migrate(db: DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE TABLE IF NOT EXISTS projects SCHEMALESS;
      DEFINE TABLE IF NOT EXISTS sessions SCHEMALESS;
      DEFINE TABLE IF NOT EXISTS messages SCHEMALESS;
      "#,
    )
    .await?;

    let _ = ProviderModelV2::migrate(&db).await;
    let _ = ProjectModelV2::migrate(&db).await;

    Ok(())
  }
}

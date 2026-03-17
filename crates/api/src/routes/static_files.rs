use std::env;
use std::path::PathBuf;

use axum::Router;
use axum::routing::get_service;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

pub fn routes() -> Router {
  let static_base_dir = env::var("BLPRNT_BASE_DIR").unwrap_or_else(|_| "./dist".to_string());

  let static_base_path = PathBuf::from(&static_base_dir);
  let index_file_path = static_base_path.join("index.html");

  assert!(index_file_path.exists(), "missing SPA entrypoint: {}", index_file_path.display());

  let assets_service = get_service(ServeDir::new(&static_base_path));
  let index_service = get_service(ServeFile::new(index_file_path));

  Router::new().nest_service("/assets", assets_service).fallback_service(index_service)
}

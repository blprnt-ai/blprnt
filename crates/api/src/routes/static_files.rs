use std::env;
use std::path::PathBuf;

use axum::Router;
use axum::routing::get_service;
use shared::paths::executable_dir;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

pub fn routes() -> Router {
  let static_base_path = resolve_static_base_dir();
  let index_file_path = static_base_path.join("index.html");
  let assets_dir_path = static_base_path.join("assets");
  let fonts_dir_path = static_base_path.join("fonts");
  let images_dir_path = static_base_path.join("images");
  let sounds_dir_path = static_base_path.join("sounds");

  assert!(index_file_path.exists(), "missing SPA entrypoint: {}", index_file_path.display());

  let assets_service = get_service(ServeDir::new(assets_dir_path));
  let fonts_service = get_service(ServeDir::new(fonts_dir_path));
  let images_service = get_service(ServeDir::new(images_dir_path));
  let sounds_service = get_service(ServeDir::new(sounds_dir_path));
  let index_service = get_service(ServeFile::new(index_file_path));
  let manifest_service = get_service(ServeFile::new(static_base_path.join("manifest.json")));
  let robots_service = get_service(ServeFile::new(static_base_path.join("robots.txt")));

  Router::new()
    .nest_service("/assets", assets_service)
    .nest_service("/fonts", fonts_service)
    .nest_service("/images", images_service)
    .nest_service("/sounds", sounds_service)
    .route_service("/manifest.json", manifest_service)
    .route_service("/robots.txt", robots_service)
    .fallback_service(index_service)
}

fn resolve_static_base_dir() -> PathBuf {
  if let Ok(static_base_dir) = env::var("BLPRNT_BASE_DIR") {
    return PathBuf::from(static_base_dir);
  }

  if let Some(executable_dist_dir) = executable_dir().map(|dir| dir.join("dist"))
    && executable_dist_dir.is_dir()
  {
    return executable_dist_dir;
  }

  PathBuf::from("./dist")
}

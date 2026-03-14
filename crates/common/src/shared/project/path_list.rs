use std::path::PathBuf;

use surrealdb_types::SurrealValue;

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
#[serde(transparent)]
pub struct PathList(pub Vec<String>);

impl PathList {
  pub fn into_vec(&self) -> Vec<PathBuf> {
    self.0.clone().into_iter().map(PathBuf::from).collect()
  }
}

impl From<Vec<PathBuf>> for PathList {
  fn from(path_list: Vec<PathBuf>) -> Self {
    PathList(path_list.into_iter().map(|path| path.to_string_lossy().to_string()).collect())
  }
}

impl From<PathList> for Vec<PathBuf> {
  fn from(path_list: PathList) -> Self {
    path_list.0.into_iter().map(PathBuf::from).collect()
  }
}

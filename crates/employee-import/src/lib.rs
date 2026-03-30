use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeProviderConfig;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::EmployeeRuntimeConfig;
use persistence::prelude::EmployeeSkillRef;
use tempfile::TempDir;

pub const DEFAULT_EMPLOYEES_REPO_URL: &str = "https://github.com/blprnt-ai/employees";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmployeeLibrarySource {
  Local(PathBuf),
  GitUrl(String),
}

#[derive(Clone, Debug)]
pub struct ImportEmployeeRequest {
  pub slug: String,
  pub source: EmployeeLibrarySource,
  pub workspace_root: PathBuf,
  pub reports_to: Option<EmployeeId>,
  pub force: bool,
}

#[derive(Clone, Debug)]
pub struct ImportEmployeeResult {
  pub action: ImportEmployeeAction,
  pub employee: EmployeeRecord,
  pub employee_home: PathBuf,
  pub installed_skills: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportEmployeeAction {
  Created,
  Updated,
}

#[derive(Debug, serde::Deserialize)]
struct EmployeeManifest {
  name: String,
  role: String,
  capabilities: Vec<String>,
  #[serde(default)]
  skills: Vec<String>,
}

pub async fn import_employee(request: ImportEmployeeRequest) -> Result<ImportEmployeeResult> {
  let checkout = checkout_source(&request.source)?;
  let repo_root = checkout.repo_root();
  let employee_dir = repo_root.join("employees").join(&request.slug);
  anyhow::ensure!(
    employee_dir.is_dir(),
    "employee definition not found for slug {} in {}",
    request.slug,
    repo_root.display()
  );

  let manifest = load_manifest(&employee_dir)?;
  let installed_skills = install_skills(repo_root, &manifest.skills)?;
  let skill_stack = build_skill_stack(&installed_skills)?;
  let template = employee_template().await?;
  let role = parse_role(&manifest.role)?;

  let (action, employee) = if role.is_ceo() {
    import_ceo(&request, &manifest, template.as_ref(), skill_stack).await?
  } else {
    (ImportEmployeeAction::Created, create_employee(&request, &manifest, template.as_ref(), role, skill_stack).await?)
  };

  let employee_home = shared::paths::employee_home(&request.workspace_root, &employee.id.uuid().to_string());
  install_employee_files(&employee_dir, &employee_home)?;

  Ok(ImportEmployeeResult { action, employee, employee_home, installed_skills })
}

fn checkout_source(source: &EmployeeLibrarySource) -> Result<CheckoutSource> {
  match source {
    EmployeeLibrarySource::Local(path) => Ok(CheckoutSource { root: path.clone(), _temp_dir: None }),
    EmployeeLibrarySource::GitUrl(url) => {
      let temp_dir = TempDir::new().context("failed to create temporary checkout directory")?;
      let repo_root = temp_dir.path().join("repo");
      let output = Command::new("git")
        .args(["clone", "--depth", "1", url, repo_root.to_string_lossy().as_ref()])
        .output()
        .with_context(|| format!("failed to execute git clone for {url}"))?;

      if !output.status.success() {
        anyhow::bail!("failed to clone employees repo {}: {}", url, String::from_utf8_lossy(&output.stderr).trim());
      }

      Ok(CheckoutSource { root: repo_root, _temp_dir: Some(temp_dir) })
    }
  }
}

fn load_manifest(employee_dir: &Path) -> Result<EmployeeManifest> {
  let manifest_path = employee_dir.join("blprnt.yml");
  let manifest =
    fs::read_to_string(&manifest_path).with_context(|| format!("failed to read {}", manifest_path.display()))?;
  serde_yaml::from_str(&manifest).with_context(|| format!("failed to parse {}", manifest_path.display()))
}

fn install_skills(repo_root: &Path, skill_names: &[String]) -> Result<Vec<PathBuf>> {
  let mut installed = Vec::new();
  for skill_name in skill_names {
    let installed_path = install_skill(repo_root, skill_name)?;
    installed.push(installed_path);
  }

  Ok(installed)
}

fn install_skill(repo_root: &Path, skill_name: &str) -> Result<PathBuf> {
  let repo_skill_dir = repo_root.join("skills").join(skill_name);
  if repo_skill_dir.join("SKILL.md").is_file() {
    let target_dir = shared::paths::agents_skills_dir().join(skill_name);
    if target_dir.exists() {
      remove_path(&target_dir)?;
    }
    copy_dir_all(&repo_skill_dir, &target_dir)?;
    let metadata = skills::validate_skill_path(&target_dir.join("SKILL.md"), Some(skill_name))?;
    return Ok(metadata.path);
  }

  if let Ok(existing) = skills::list_skills()
    && let Some(skill) = existing.into_iter().find(|skill| skill.name == skill_name)
  {
    return Ok(skill.path);
  }

  anyhow::bail!("skill {} not found in employees repo or installed skill roots", skill_name)
}

fn build_skill_stack(installed_skills: &[PathBuf]) -> Result<Option<Vec<EmployeeSkillRef>>> {
  if installed_skills.is_empty() {
    return Ok(None);
  }

  let mut skill_stack = Vec::with_capacity(installed_skills.len());
  for path in installed_skills {
    let metadata = skills::validate_skill_path(path, None)?;
    skill_stack.push(EmployeeSkillRef { name: metadata.name, path: metadata.path.to_string_lossy().to_string() });
  }

  Ok(Some(skill_stack))
}

async fn employee_template() -> Result<Option<EmployeeTemplate>> {
  let ceo = EmployeeRepository::list().await?.into_iter().find(|employee| employee.role.is_ceo());
  Ok(ceo.map(|employee| EmployeeTemplate {
    provider_config: employee.provider_config.or_else(|| Some(EmployeeProviderConfig::default())),
    runtime_config: employee.runtime_config.or_else(|| Some(default_runtime_config(None))),
  }))
}

async fn import_ceo(
  request: &ImportEmployeeRequest,
  manifest: &EmployeeManifest,
  template: Option<&EmployeeTemplate>,
  skill_stack: Option<Vec<EmployeeSkillRef>>,
) -> Result<(ImportEmployeeAction, EmployeeRecord)> {
  let existing_ceo = EmployeeRepository::list().await?.into_iter().find(|employee| employee.role.is_ceo());
  match existing_ceo {
    Some(_existing) if !request.force => {
      anyhow::bail!("CEO already exists; re-run with --force to overwrite the existing CEO")
    }
    Some(existing) => {
      let runtime_config = templated_runtime_config(template, skill_stack);
      let provider_config = templated_provider_config(template);
      let employee = EmployeeRepository::update(
        existing.id,
        EmployeePatch {
          name: Some(manifest.name.clone()),
          role: Some(EmployeeRole::Ceo),
          title: Some(manifest.name.clone()),
          icon: Some("bot".to_string()),
          color: Some("gray".to_string()),
          capabilities: Some(manifest.capabilities.clone()),
          reports_to: Some(request.reports_to.clone()),
          provider_config,
          runtime_config,
          ..Default::default()
        },
      )
      .await
      .context("failed to overwrite existing CEO")?;
      Ok((ImportEmployeeAction::Updated, employee))
    }
    None => Ok((
      ImportEmployeeAction::Created,
      create_employee(request, manifest, template, EmployeeRole::Ceo, skill_stack).await?,
    )),
  }
}

async fn create_employee(
  request: &ImportEmployeeRequest,
  manifest: &EmployeeManifest,
  template: Option<&EmployeeTemplate>,
  role: EmployeeRole,
  skill_stack: Option<Vec<EmployeeSkillRef>>,
) -> Result<EmployeeRecord> {
  EmployeeRepository::create(EmployeeModel {
    name: manifest.name.clone(),
    kind: EmployeeKind::Agent,
    role,
    title: manifest.name.clone(),
    icon: "bot".to_string(),
    color: "gray".to_string(),
    capabilities: manifest.capabilities.clone(),
    reports_to: request.reports_to.clone(),
    provider_config: templated_provider_config(template),
    runtime_config: templated_runtime_config(template, skill_stack),
    ..Default::default()
  })
  .await
  .context("failed to create imported employee")
}

fn templated_provider_config(template: Option<&EmployeeTemplate>) -> Option<EmployeeProviderConfig> {
  template.and_then(|template| template.provider_config.clone()).or_else(|| Some(EmployeeProviderConfig::default()))
}

fn templated_runtime_config(
  template: Option<&EmployeeTemplate>,
  skill_stack: Option<Vec<EmployeeSkillRef>>,
) -> Option<EmployeeRuntimeConfig> {
  let mut runtime_config = template
    .and_then(|template| template.runtime_config.clone())
    .unwrap_or_else(|| default_runtime_config(skill_stack.clone()));
  runtime_config.skill_stack = skill_stack;
  Some(runtime_config)
}

fn default_runtime_config(skill_stack: Option<Vec<EmployeeSkillRef>>) -> EmployeeRuntimeConfig {
  EmployeeRuntimeConfig {
    heartbeat_interval_sec: 1800,
    heartbeat_prompt: String::new(),
    wake_on_demand: true,
    max_concurrent_runs: 1,
    skill_stack,
  }
}

fn parse_role(role: &str) -> Result<EmployeeRole> {
  match role {
    "ceo" => Ok(EmployeeRole::Ceo),
    "manager" => Ok(EmployeeRole::Manager),
    "staff" => Ok(EmployeeRole::Staff),
    other => Ok(EmployeeRole::Custom(other.to_string())),
  }
}

fn install_employee_files(source_dir: &Path, employee_home: &Path) -> Result<()> {
  fs::create_dir_all(employee_home).with_context(|| format!("failed to create {}", employee_home.display()))?;
  for file_name in ["AGENTS.md", "HEARTBEAT.md", "SOUL.md", "TOOLS.md", "blprnt.yml"] {
    let source = source_dir.join(file_name);
    if !source.is_file() {
      continue;
    }
    let target = employee_home.join(file_name);
    fs::copy(&source, &target)
      .with_context(|| format!("failed to copy {} to {}", source.display(), target.display()))?;
  }
  Ok(())
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
  fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
  for entry in fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let source_path = entry.path();
    let target_path = target.join(entry.file_name());
    if file_type.is_dir() {
      copy_dir_all(&source_path, &target_path)?;
    } else {
      if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
      }
      fs::copy(&source_path, &target_path)
        .with_context(|| format!("failed to copy {} to {}", source_path.display(), target_path.display()))?;
    }
  }

  Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
  let metadata = fs::symlink_metadata(path).with_context(|| format!("failed to inspect {}", path.display()))?;
  if metadata.file_type().is_symlink() || metadata.is_file() {
    fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
  } else {
    fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
  }
  Ok(())
}

struct CheckoutSource {
  root: PathBuf,
  _temp_dir: Option<TempDir>,
}

impl CheckoutSource {
  fn repo_root(&self) -> &Path {
    &self.root
  }
}

#[derive(Clone)]
struct EmployeeTemplate {
  provider_config: Option<EmployeeProviderConfig>,
  runtime_config: Option<EmployeeRuntimeConfig>,
}

#[cfg(test)]
mod tests {
  use std::fs;

  use persistence::prelude::DbId;
  use persistence::prelude::EmployeeKind;
  use persistence::prelude::EmployeeRole;
  use persistence::prelude::SurrealConnection;
  use shared::agent::Provider;
  use tempfile::TempDir;

  use super::*;

  static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
  static TEST_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("failed to create test runtime")
  });

  struct HomeGuard {
    previous_home: Option<String>,
  }

  impl HomeGuard {
    fn set(temp_home: &TempDir) -> Self {
      let previous_home = std::env::var("HOME").ok();
      unsafe { std::env::set_var("HOME", temp_home.path()) };
      Self { previous_home }
    }
  }

  impl Drop for HomeGuard {
    fn drop(&mut self) {
      match &self.previous_home {
        Some(home) => unsafe { std::env::set_var("HOME", home) },
        None => unsafe { std::env::remove_var("HOME") },
      }
    }
  }

  struct CwdGuard {
    previous_cwd: std::path::PathBuf,
  }

  impl CwdGuard {
    fn set(path: &std::path::Path) -> Self {
      let previous_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(path).unwrap();
      Self { previous_cwd }
    }
  }

  impl Drop for CwdGuard {
    fn drop(&mut self) {
      std::env::set_current_dir(&self.previous_cwd).unwrap();
    }
  }

  fn write_employee_repo(root: &std::path::Path) {
    let employee_dir = root.join("employees").join("data-analyst");
    fs::create_dir_all(&employee_dir).unwrap();
    fs::write(
      employee_dir.join("blprnt.yml"),
      "name: Data Analyst\nrole: staff\ncapabilities:\n  - reporting\nskills:\n  - analytics-tracking\n",
    )
    .unwrap();
    fs::write(employee_dir.join("AGENTS.md"), "You are the Data Analyst.\n").unwrap();
    fs::write(employee_dir.join("HEARTBEAT.md"), "Check dashboards.\n").unwrap();
    fs::write(employee_dir.join("SOUL.md"), "Stay evidence-driven.\n").unwrap();
    fs::write(employee_dir.join("TOOLS.md"), "Use product analytics.\n").unwrap();

    let skill_dir = root.join("skills").join("analytics-tracking");
    fs::create_dir_all(skill_dir.join("references")).unwrap();
    fs::write(
      skill_dir.join("SKILL.md"),
      "---\nname: analytics-tracking\ndescription: Analyze product analytics.\n---\n\n# Analytics Tracking\n",
    )
    .unwrap();
    fs::write(skill_dir.join("references").join("events.md"), "Track the right events.\n").unwrap();
  }

  #[test]
  fn import_employee_creates_record_and_installs_assets() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    TEST_RUNTIME.block_on(async {
      let home = TempDir::new().unwrap();
      let repo = TempDir::new().unwrap();
      let workspace = TempDir::new().unwrap();
      let _home_guard = HomeGuard::set(&home);
      let _cwd_guard = CwdGuard::set(workspace.path());
      write_employee_repo(repo.path());
      let _ = SurrealConnection::reset().await;

      let imported = import_employee(ImportEmployeeRequest {
        slug: "data-analyst".to_string(),
        source: EmployeeLibrarySource::Local(repo.path().to_path_buf()),
        workspace_root: workspace.path().to_path_buf(),
        reports_to: None,
        force: false,
      })
      .await
      .unwrap();

      assert_eq!(imported.employee.name, "Data Analyst");
      assert!(matches!(imported.employee.role, EmployeeRole::Staff));
      assert_eq!(imported.employee.capabilities, vec!["reporting"]);
      assert_eq!(imported.installed_skills.len(), 1);
      let provider_config = imported.employee.provider_config.as_ref().expect("provider config");
      assert_eq!(provider_config.provider, Provider::Mock);
      assert!(provider_config.slug.is_empty());

      let skill_path = shared::paths::agents_skills_dir().join("analytics-tracking").join("SKILL.md");
      assert!(skill_path.is_file(), "expected installed skill at {}", skill_path.display());
      assert_eq!(
        imported
          .installed_skills
          .iter()
          .map(|path| fs::canonicalize(path).unwrap())
          .collect::<Vec<_>>(),
        vec![fs::canonicalize(&skill_path).unwrap()]
      );

      let employee_home = shared::paths::employee_home(workspace.path(), &imported.employee.id.uuid().to_string());
      assert_eq!(imported.employee_home, employee_home);
      assert_eq!(fs::read_to_string(employee_home.join("AGENTS.md")).unwrap(), "You are the Data Analyst.\n");

      let skill_stack = imported.employee.runtime_config.as_ref().and_then(|config| config.skill_stack.clone()).unwrap();
      assert_eq!(skill_stack.len(), 1);
      assert_eq!(skill_stack[0].name, "analytics-tracking");
      assert_eq!(fs::canonicalize(PathBuf::from(&skill_stack[0].path)).unwrap(), fs::canonicalize(skill_path).unwrap());
      assert_eq!(imported.employee.runtime_config.as_ref().unwrap().heartbeat_interval_sec, 1800);
      assert!(imported.employee.runtime_config.as_ref().unwrap().wake_on_demand);
      assert_eq!(imported.employee.runtime_config.as_ref().unwrap().max_concurrent_runs, 1);
      assert_eq!(imported.employee.reports_to, None);
    });
  }

  #[test]
  fn import_employee_copies_provider_and_runtime_from_existing_ceo() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    TEST_RUNTIME.block_on(async {
      let home = TempDir::new().unwrap();
      let repo = TempDir::new().unwrap();
      let workspace = TempDir::new().unwrap();
      let _home_guard = HomeGuard::set(&home);
      let _cwd_guard = CwdGuard::set(workspace.path());
      write_employee_repo(repo.path());
      let _ = SurrealConnection::reset().await;

      EmployeeRepository::create(EmployeeModel {
        name: "CEO".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Ceo,
        title: "CEO".to_string(),
        provider_config: Some(EmployeeProviderConfig { provider: Provider::Codex, slug: "gpt-5.4".to_string() }),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 900,
          heartbeat_prompt: "Lead the company.".to_string(),
          wake_on_demand: false,
          max_concurrent_runs: 3,
          skill_stack: None,
        }),
        ..Default::default()
      })
      .await
      .unwrap();

      let imported = import_employee(ImportEmployeeRequest {
        slug: "data-analyst".to_string(),
        source: EmployeeLibrarySource::Local(repo.path().to_path_buf()),
        workspace_root: workspace.path().to_path_buf(),
        reports_to: None,
        force: false,
      })
      .await
      .unwrap();

      let provider_config = imported.employee.provider_config.as_ref().expect("provider config");
      assert_eq!(provider_config.provider, Provider::Codex);
      assert_eq!(provider_config.slug, "gpt-5.4");
      let runtime = imported.employee.runtime_config.expect("runtime config");
      assert_eq!(runtime.heartbeat_interval_sec, 900);
      assert_eq!(runtime.heartbeat_prompt, "Lead the company.");
      assert!(!runtime.wake_on_demand);
      assert_eq!(runtime.max_concurrent_runs, 3);
      assert_eq!(runtime.skill_stack.unwrap().len(), 1);
    });
  }

  #[test]
  fn import_employee_rejects_existing_ceo_without_force() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    TEST_RUNTIME.block_on(async {
      let home = TempDir::new().unwrap();
      let repo = TempDir::new().unwrap();
      let workspace = TempDir::new().unwrap();
      let _home_guard = HomeGuard::set(&home);
      let _cwd_guard = CwdGuard::set(workspace.path());

      let employee_dir = repo.path().join("employees").join("ceo");
      fs::create_dir_all(&employee_dir).unwrap();
      fs::write(employee_dir.join("blprnt.yml"), "name: CEO\nrole: ceo\ncapabilities:\n  - strategy\n").unwrap();
      fs::write(employee_dir.join("AGENTS.md"), "You are the CEO.\n").unwrap();
      fs::write(employee_dir.join("HEARTBEAT.md"), "Drive the company.\n").unwrap();
      fs::write(employee_dir.join("SOUL.md"), "Think long term.\n").unwrap();
      fs::write(employee_dir.join("TOOLS.md"), "Use the API.\n").unwrap();
      let _ = SurrealConnection::reset().await;

      EmployeeRepository::create(EmployeeModel {
        name: "Existing CEO".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Ceo,
        title: "CEO".to_string(),
        ..Default::default()
      })
      .await
      .unwrap();

      let error = import_employee(ImportEmployeeRequest {
        slug: "ceo".to_string(),
        source: EmployeeLibrarySource::Local(repo.path().to_path_buf()),
        workspace_root: workspace.path().to_path_buf(),
        reports_to: None,
        force: false,
      })
      .await
      .unwrap_err();

      assert!(error.to_string().contains("--force"));
    });
  }

  #[test]
  fn import_employee_force_updates_existing_ceo() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    TEST_RUNTIME.block_on(async {
      let home = TempDir::new().unwrap();
      let repo = TempDir::new().unwrap();
      let workspace = TempDir::new().unwrap();
      let _home_guard = HomeGuard::set(&home);
      let _cwd_guard = CwdGuard::set(workspace.path());

      let employee_dir = repo.path().join("employees").join("ceo");
      fs::create_dir_all(&employee_dir).unwrap();
      fs::write(employee_dir.join("blprnt.yml"), "name: CEO\nrole: ceo\ncapabilities:\n  - strategy\n").unwrap();
      fs::write(employee_dir.join("AGENTS.md"), "You are the CEO.\n").unwrap();
      fs::write(employee_dir.join("HEARTBEAT.md"), "Drive the company.\n").unwrap();
      fs::write(employee_dir.join("SOUL.md"), "Think long term.\n").unwrap();
      fs::write(employee_dir.join("TOOLS.md"), "Use the API.\n").unwrap();
      let _ = SurrealConnection::reset().await;

      let existing = EmployeeRepository::create(EmployeeModel {
        name: "Existing CEO".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Ceo,
        title: "CEO".to_string(),
        ..Default::default()
      })
      .await
      .unwrap();

      let imported = import_employee(ImportEmployeeRequest {
        slug: "ceo".to_string(),
        source: EmployeeLibrarySource::Local(repo.path().to_path_buf()),
        workspace_root: workspace.path().to_path_buf(),
        reports_to: None,
        force: true,
      })
      .await
      .unwrap();

      assert_eq!(imported.employee.id, existing.id);
      assert_eq!(imported.employee.name, "CEO");
    });
  }

  #[test]
  fn import_employee_errors_when_employee_definition_is_missing() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    TEST_RUNTIME.block_on(async {
      let home = TempDir::new().unwrap();
      let repo = TempDir::new().unwrap();
      let workspace = TempDir::new().unwrap();
      let _home_guard = HomeGuard::set(&home);
      let _cwd_guard = CwdGuard::set(workspace.path());
      let _ = SurrealConnection::reset().await;

      let error = import_employee(ImportEmployeeRequest {
        slug: "missing-employee".to_string(),
        source: EmployeeLibrarySource::Local(repo.path().to_path_buf()),
        workspace_root: workspace.path().to_path_buf(),
        reports_to: None,
        force: false,
      })
      .await
      .unwrap_err();

      assert!(error.to_string().contains("missing-employee"));
    });
  }
}

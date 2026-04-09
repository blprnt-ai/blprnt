use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveDate;
use chrono::Utc;
use events::API_EVENTS;
use events::ApiEvent;
use events::COORDINATOR_EVENTS;
use events::CoordinatorEvent;
use events::MinionRunKind;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeStatus;
use persistence::prelude::RunFilter;
use persistence::prelude::RunId;
use persistence::prelude::RunModel;
use persistence::prelude::RunRecord;
use persistence::prelude::RunRepository;
use persistence::prelude::RunStatus;
use persistence::prelude::RunTrigger;
use shared::errors::CoordinatorError;
use shared::errors::CoordinatorResult;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::state::EmployeeRuntimeState;

struct EmployeeScheduleEntry {
  scheduler_cancel_token: CancellationToken,
  runtime_state:          Arc<EmployeeRuntimeState>,
}

#[derive(Clone, Copy)]
enum MissingLastRunPolicy {
  RunNow,
  WaitHeartbeat,
}

#[derive(Clone, Copy)]
enum SchedulerStartMode {
  Bootstrap { position: usize },
  Live,
}

pub struct Coordinator {
  schedules: Arc<RwLock<HashMap<EmployeeId, EmployeeScheduleEntry>>>,
}

impl Coordinator {
  const BOOT_RUN_STAGGER_MS: u64 = 5_000;
  const INTERRUPTED_RUN_REASON: &'static str = "system interrupted";

  pub fn new() -> Arc<Self> {
    Arc::new(Self { schedules: Arc::new(RwLock::new(HashMap::new())) })
  }

  pub async fn init(self: &Arc<Self>) -> CoordinatorResult<()> {
    RunRepository::mark_all_pending_as_failed("system shutdown".to_string())
      .await
      .map_err(CoordinatorError::DatabaseError)?;

    let interrupted_employee_ids = Self::fail_all_interrupted_runs().await?;
    let mut employees = EmployeeRepository::list_agents().await.map_err(CoordinatorError::DatabaseError)?;

    employees.retain(|e| e.kind.is_agent());

    for employee in &employees {
      if employee.status == EmployeeStatus::Running || interrupted_employee_ids.contains(&employee.id) {
        Self::mark_employee_idle(&employee.id).await?;
      }
    }

    for (position, employee) in employees.into_iter().enumerate() {
      self.upsert_employee(employee.id.clone(), SchedulerStartMode::Bootstrap { position }).await;
    }

    Ok(())
  }

  pub async fn listen(self: Arc<Self>) {
    let mut rx = API_EVENTS.subscribe();

    loop {
      let event = rx.recv().await;

      match event {
        Ok(event) => match event {
          ApiEvent::StartRun { employee_id, run_id, trigger, rx } => {
            let run = self.trigger_run_now(&employee_id, run_id, trigger).await;
            if let Some(rx) = rx
              && let Some(rx) = rx.lock().await.take()
            {
              let _ = rx.send(run.map_err(Into::into));
            }
          }
          ApiEvent::AddEmployee { employee_id } | ApiEvent::UpdateEmployee { employee_id } => {
            self.upsert_employee(employee_id, SchedulerStartMode::Live).await;
          }
          ApiEvent::DeleteEmployee { employee_id } => {
            self.remove_employee(&employee_id).await;
          }
          ApiEvent::CancelRun { employee_id, run_id } => {
            self.cancel_run(&employee_id, &run_id).await;
          }
        },
        Err(error) => {
          tracing::error!(?error, "failed to receive event");
          continue;
        }
      }
    }
  }

  async fn trigger_run_now(
    self: &Arc<Self>,
    employee_id: &EmployeeId,
    existing_run_id: Option<RunId>,
    run_trigger: RunTrigger,
  ) -> CoordinatorResult<Option<RunRecord>> {
    if matches!(run_trigger, RunTrigger::Dreaming) {
      return Err(CoordinatorError::MinionOnlyTrigger);
    }

    let employee = Self::load_employee(employee_id).await?;
    let bypass_guards = matches!(run_trigger, RunTrigger::Conversation);

    if !bypass_guards && employee.status == EmployeeStatus::Paused {
      tracing::info!("Employee {:?} is paused, skipping run", employee_id);
      return Err(CoordinatorError::EmployeePaused);
    }

    if matches!(run_trigger, RunTrigger::IssueAssignment { .. } | RunTrigger::IssueMention { .. })
      && !employee.runtime_config.as_ref().map(|config| config.wake_on_demand).unwrap_or(false)
    {
      return Ok(None);
    }

    let runtime_state = {
      let schedules = self.schedules.read().await;

      schedules.get(employee_id).map(|entry| entry.runtime_state.clone()).ok_or(CoordinatorError::EmployeeNotManaged)?
    };

    let max_concurrent_runs =
      employee.runtime_config.as_ref().map(|config| config.max_concurrent_runs.max(1) as usize).unwrap_or(1);

    let counted_slot = !bypass_guards;

    if counted_slot && !runtime_state.try_reserve_slot(max_concurrent_runs) {
      return Err(CoordinatorError::NoRunSlotsAvailable);
    }

    if let Err(error) = Self::mark_employee_started(employee_id, counted_slot).await {
      tracing::error!(?employee_id, ?error, "failed to mark employee started {:?}", employee_id);
      let remaining_count = counted_slot.then(|| runtime_state.release_slot());

      if remaining_count == Some(0) {
        let _ = Self::mark_employee_idle(employee_id).await;
      }

      return Err(error);
    }

    let run = match existing_run_id {
      Some(run_id) => RunRepository::get(run_id).await.map_err(CoordinatorError::DatabaseError)?,
      None => RunRepository::create(RunModel::new(employee_id.clone(), run_trigger))
        .await
        .map_err(CoordinatorError::DatabaseError)?,
    };

    self.spawn_employee_run(run.clone(), runtime_state, counted_slot);

    Ok(Some(run))
  }

  async fn trigger_minion_dream_now(self: &Arc<Self>, employee_id: &EmployeeId) -> CoordinatorResult<RunRecord> {
    let employee = Self::load_employee(employee_id).await?;

    let runtime_state = {
      let schedules = self.schedules.read().await;

      schedules.get(employee_id).map(|entry| entry.runtime_state.clone()).ok_or(CoordinatorError::EmployeeNotManaged)?
    };

    self.spawn_minion_dream_run(employee.id.clone(), runtime_state, false);

    Ok(RunRecord {
      id:           RunId::from(Uuid::new_v4()),
      employee_id:  employee.id,
      status:       RunStatus::Pending,
      trigger:      RunTrigger::Dreaming,
      turns:        vec![],
      usage:        None,
      created_at:   Utc::now(),
      started_at:   None,
      completed_at: None,
    })
  }

  async fn cancel_run(self: &Arc<Self>, employee_id: &EmployeeId, run_id: &RunId) {
    let schedules = self.schedules.read().await;

    if let Some(entry) = schedules.get(employee_id) {
      entry.runtime_state.cancel_run(run_id).await;
    }
  }

  async fn upsert_employee(self: &Arc<Self>, employee_id: EmployeeId, start_mode: SchedulerStartMode) {
    let runtime_state = {
      let mut schedules = self.schedules.write().await;

      match schedules.remove(&employee_id) {
        Some(existing) => {
          existing.scheduler_cancel_token.cancel();
          existing.runtime_state
        }
        None => Arc::new(EmployeeRuntimeState::new()),
      }
    };

    if !Self::load_employee(&employee_id).await.is_err() {
      let scheduler_cancel_token = CancellationToken::new();

      self.schedules.write().await.insert(
        employee_id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: scheduler_cancel_token.clone(),
          runtime_state:          runtime_state.clone(),
        },
      );

      tokio::spawn(self.clone().employee_scheduler_loop(
        employee_id,
        runtime_state,
        scheduler_cancel_token,
        start_mode,
      ));
    }
  }

  async fn remove_employee(&self, employee_id: &EmployeeId) {
    if let Some(existing) = self.schedules.write().await.remove(employee_id) {
      existing.scheduler_cancel_token.cancel();
    }
  }

  async fn employee_scheduler_loop(
    self: Arc<Self>,
    employee_id: EmployeeId,
    runtime_state: Arc<EmployeeRuntimeState>,
    scheduler_cancel_token: CancellationToken,
    start_mode: SchedulerStartMode,
  ) {
    let mut is_first_active_iteration = true;

    loop {
      let employee = match Self::load_employee(&employee_id).await {
        Ok(employee) => employee,
        Err(error) => {
          tracing::error!(?employee_id, ?error, "failed to load employee");
          if !Self::wait_or_cancel(Duration::from_secs(30), &scheduler_cancel_token).await {
            break;
          }
          continue;
        }
      };

      if employee.status == EmployeeStatus::Paused {
        if !Self::wait_or_cancel(Duration::from_secs(30), &scheduler_cancel_token).await {
          break;
        }
        continue;
      }

      if !employee.runtime_config.as_ref().map(|config| config.timer_wakeups_enabled()).unwrap_or(true) {
        if !Self::wait_or_cancel(Duration::from_secs(30), &scheduler_cancel_token).await {
          break;
        }
        continue;
      }

      let sleep_duration = if is_first_active_iteration {
        is_first_active_iteration = false;
        Self::initial_sleep_duration(&employee, start_mode)
      } else {
        Self::next_due_duration(&employee, MissingLastRunPolicy::RunNow)
      };

      if !Self::wait_or_cancel(sleep_duration, &scheduler_cancel_token).await {
        break;
      }

      let employee = match Self::load_employee(&employee_id).await {
        Ok(employee) => employee,
        Err(error) => {
          tracing::error!(?employee_id, ?error, "failed to reload employee");
          if !Self::wait_or_cancel(Duration::from_secs(30), &scheduler_cancel_token).await {
            break;
          }
          continue;
        }
      };

      if employee.status == EmployeeStatus::Paused {
        continue;
      }

      if !employee.runtime_config.as_ref().map(|config| config.timer_wakeups_enabled()).unwrap_or(true) {
        continue;
      }

      let Ok(run) = RunRepository::create(RunModel::new(employee_id.clone(), RunTrigger::Timer)).await else {
        tracing::error!(?employee_id, "failed to create run");
        continue;
      };

      let max_concurrent_runs =
        employee.runtime_config.as_ref().map(|config| config.max_concurrent_runs.max(1) as usize).unwrap_or(1);

      if !Self::wait_for_capacity(runtime_state.clone(), max_concurrent_runs, &scheduler_cancel_token).await {
        break;
      }

      if let Err(error) = Self::mark_employee_started(&employee_id, true).await {
        let remaining_count = runtime_state.release_slot();

        if remaining_count == 0 {
          let _ = Self::mark_employee_idle(&employee_id).await;
        }

        tracing::error!(?employee_id, ?error, "failed to mark employee started");

        if !Self::wait_or_cancel(Duration::from_secs(10), &scheduler_cancel_token).await {
          break;
        }

        continue;
      }

      let run_runtime_state = runtime_state.clone();

      self.spawn_employee_run(run, run_runtime_state, true);

      if employee.runtime_config.as_ref().map(|config| config.dreams_enabled()).unwrap_or(false)
        && !Self::has_dreaming_run_for_today(&employee_id).await
      {
        if let Err(error) = self.trigger_minion_dream_now(&employee_id).await {
          tracing::error!(?employee_id, ?error, "failed to start dreaming minion run");
          continue;
        }
      }
    }
  }

  async fn has_dreaming_run_for_today(employee_id: &EmployeeId) -> bool {
    let today = Utc::now().date_naive();
    Self::has_dreaming_stamp_for_date(employee_id, today)
  }

  fn dreaming_stamp_path(employee_id: &EmployeeId, date: NaiveDate) -> std::path::PathBuf {
    shared::paths::employee_home(&employee_id.uuid().to_string())
      .join("memory")
      .join("dreaming")
      .join(format!("{}.stamp", date.format("%Y-%m-%d")))
  }

  fn has_dreaming_stamp_for_date(employee_id: &EmployeeId, date: NaiveDate) -> bool {
    fs::metadata(Self::dreaming_stamp_path(employee_id, date)).is_ok()
  }

  fn spawn_employee_run(
    self: &Arc<Self>,
    run: RunRecord,
    runtime_state: Arc<EmployeeRuntimeState>,
    counted_slot: bool,
  ) {
    let coordinator = self.clone();

    tokio::spawn(async move {
      let employee_id = run.employee_id.clone();
      let run_id = run.id.clone();
      let run_result = coordinator.run_employee_once(run, runtime_state.clone(), counted_slot).await;

      if let Err(error) = run_result {
        tracing::error!(?employee_id, ?error, "employee run failed");
      }

      let Some(remaining_count) = runtime_state.finish_run(&run_id).await else {
        tracing::info!("No remaining count for employee {:?}", employee_id);
        return;
      };

      if counted_slot
        && remaining_count == 0
        && let Err(error) = Self::mark_employee_idle(&employee_id).await
      {
        tracing::error!(?employee_id, ?error, "failed to mark employee idle");
      }
    });
  }

  fn spawn_minion_dream_run(
    self: &Arc<Self>,
    employee_id: EmployeeId,
    runtime_state: Arc<EmployeeRuntimeState>,
    counted_slot: bool,
  ) {
    let coordinator = self.clone();
    let synthetic_run_id = RunId::from(Uuid::new_v4());

    tokio::spawn(async move {
      let run_result = coordinator
        .run_minion_dream_once(employee_id.clone(), synthetic_run_id.clone(), runtime_state.clone(), counted_slot)
        .await;

      if let Err(error) = run_result {
        tracing::error!(?employee_id, ?error, "minion dream run failed");
      }

      let Some(remaining_count) = runtime_state.finish_run(&synthetic_run_id).await else {
        tracing::info!("No remaining count for employee {:?}", employee_id);
        return;
      };

      if counted_slot
        && remaining_count == 0
        && let Err(error) = Self::mark_employee_idle(&employee_id).await
      {
        tracing::error!(?employee_id, ?error, "failed to mark employee idle");
      }
    });
  }

  async fn run_employee_once(
    &self,
    run: RunRecord,
    runtime_state: Arc<EmployeeRuntimeState>,
    counted_slot: bool,
  ) -> CoordinatorResult<()> {
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<()>>();

    let run_id = run.id.clone();
    let run_cancel_token = CancellationToken::new();
    runtime_state.register_run(run_id.clone(), run_cancel_token.clone(), counted_slot).await;

    let _ = RunRepository::update(run_id.clone(), RunStatus::Running).await.map_err(CoordinatorError::DatabaseError)?;

    let handoff_result = async {
      COORDINATOR_EVENTS
        .emit(CoordinatorEvent::StartRun {
          run_id,
          cancel_token: run_cancel_token.child_token(),
          tx: Arc::new(tokio::sync::Mutex::new(Some(tx))),
        })
        .map_err(CoordinatorError::FailedToEmitCoordinatorEvent)?;

      let adapter_result = rx.await.map_err(CoordinatorError::FailedToAwaitOneshotChannel)?;
      adapter_result.map_err(CoordinatorError::AdapterRuntimeFailed)?;

      Ok(())
    }
    .await;

    if let Err(error) = &handoff_result {
      Self::fail_run_if_still_running(&run.id, error).await;
    }

    handoff_result
  }

  async fn run_minion_dream_once(
    &self,
    employee_id: EmployeeId,
    synthetic_run_id: RunId,
    runtime_state: Arc<EmployeeRuntimeState>,
    counted_slot: bool,
  ) -> CoordinatorResult<()> {
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<()>>();

    let run_cancel_token = CancellationToken::new();
    runtime_state.register_run(synthetic_run_id, run_cancel_token.clone(), counted_slot).await;

    let handoff_result = async {
      COORDINATOR_EVENTS
        .emit(CoordinatorEvent::StartMinionRun {
          employee_id,
          kind:         MinionRunKind::Dreamer,
          cancel_token: run_cancel_token.child_token(),
          tx:           Arc::new(tokio::sync::Mutex::new(Some(tx))),
        })
        .map_err(CoordinatorError::FailedToEmitCoordinatorEvent)?;

      let adapter_result = rx.await.map_err(CoordinatorError::FailedToAwaitOneshotChannel)?;
      adapter_result.map_err(CoordinatorError::AdapterRuntimeFailed)?;

      Ok(())
    }
    .await;

    handoff_result
  }

  async fn load_employee(employee_id: &EmployeeId) -> CoordinatorResult<EmployeeRecord> {
    Ok(EmployeeRepository::get(employee_id.clone()).await.map_err(CoordinatorError::DatabaseError)?)
  }

  async fn mark_employee_started(employee_id: &EmployeeId, update_status: bool) -> CoordinatorResult<()> {
    let status = update_status.then_some(EmployeeStatus::Running);
    EmployeeRepository::update(
      employee_id.clone(),
      EmployeePatch { status, last_run_at: Some(Utc::now()), ..Default::default() },
    )
    .await
    .map_err(CoordinatorError::DatabaseError)?;

    Ok(())
  }

  async fn mark_employee_idle(employee_id: &EmployeeId) -> CoordinatorResult<()> {
    let employee = Self::load_employee(employee_id).await?;
    if employee.status == EmployeeStatus::Paused {
      return Ok(());
    }

    EmployeeRepository::update(
      employee_id.clone(),
      EmployeePatch { status: Some(EmployeeStatus::Idle), ..Default::default() },
    )
    .await
    .map_err(CoordinatorError::DatabaseError)?;

    Ok(())
  }

  async fn fail_all_interrupted_runs() -> CoordinatorResult<HashSet<EmployeeId>> {
    let runs = RunRepository::list(RunFilter {
      employee: None,
      issue:    None,
      status:   Some(RunStatus::Running),
      trigger:  None,
    })
    .await
    .map_err(CoordinatorError::DatabaseError)?;
    let interrupted_employee_ids = runs.iter().map(|run| run.employee_id.clone()).collect::<HashSet<_>>();

    for run in runs {
      RunRepository::update(run.id, RunStatus::Failed(Self::INTERRUPTED_RUN_REASON.to_string()))
        .await
        .map_err(CoordinatorError::DatabaseError)?;
    }

    Ok(interrupted_employee_ids)
  }

  async fn fail_run_if_still_running(run_id: &RunId, error: &CoordinatorError) {
    let run = match RunRepository::get(run_id.clone()).await {
      Ok(run) => run,
      Err(load_error) => {
        tracing::error!(?run_id, ?load_error, "failed to load run after coordinator handoff failure");
        return;
      }
    };

    if !matches!(run.status, RunStatus::Running) {
      return;
    }

    if let Err(update_error) = RunRepository::update(run.id, RunStatus::Failed(error.to_string())).await {
      tracing::error!(?run_id, ?update_error, "failed to mark orphaned run failed after coordinator handoff failure");
    }
  }

  fn initial_sleep_duration(employee: &EmployeeRecord, start_mode: SchedulerStartMode) -> Duration {
    match start_mode {
      SchedulerStartMode::Bootstrap { position } => {
        let due_duration = Self::next_due_duration(employee, MissingLastRunPolicy::RunNow);

        if due_duration.is_zero() { Self::boot_run_stagger(position) } else { due_duration }
      }
      SchedulerStartMode::Live => Self::next_due_duration(employee, MissingLastRunPolicy::WaitHeartbeat),
    }
  }

  fn next_due_duration(employee: &EmployeeRecord, missing_last_run_policy: MissingLastRunPolicy) -> Duration {
    let heartbeat_interval = Self::heartbeat_interval_duration(employee);

    let Some(last_run_at) = employee.last_run_at else {
      return match missing_last_run_policy {
        MissingLastRunPolicy::RunNow => Duration::ZERO,
        MissingLastRunPolicy::WaitHeartbeat => heartbeat_interval,
      };
    };

    let next_run_at = last_run_at + chrono::Duration::from_std(heartbeat_interval).unwrap_or_default();
    let now = Utc::now();

    if next_run_at <= now { Duration::ZERO } else { (next_run_at - now).to_std().unwrap_or(Duration::ZERO) }
  }

  fn heartbeat_interval_duration(employee: &EmployeeRecord) -> Duration {
    let heartbeat_interval_sec =
      employee.runtime_config.as_ref().map(|config| config.heartbeat_interval_sec.max(0)).unwrap_or(3600);

    Duration::from_secs(heartbeat_interval_sec as u64)
  }

  fn boot_run_stagger(position: usize) -> Duration {
    Duration::from_millis(Self::BOOT_RUN_STAGGER_MS.saturating_mul(position as u64))
  }

  async fn wait_or_cancel(duration: Duration, scheduler_cancel_token: &CancellationToken) -> bool {
    tokio::select! {
      _ = tokio::time::sleep(duration) => true,
      _ = scheduler_cancel_token.cancelled() => false,
    }
  }

  async fn wait_for_capacity(
    runtime_state: Arc<EmployeeRuntimeState>,
    max_concurrent_runs: usize,
    scheduler_cancel_token: &CancellationToken,
  ) -> bool {
    loop {
      if runtime_state.try_reserve_slot(max_concurrent_runs) {
        return true;
      }

      tokio::select! {
        _ = runtime_state.notified() => {}
        _ = scheduler_cancel_token.cancelled() => return false,
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::sync::LazyLock;
  use std::sync::Mutex;
  use std::time::Duration;

  use chrono::Utc;
  use persistence::Uuid;
  use persistence::prelude::EmployeeId;
  use persistence::prelude::EmployeeKind;
  use persistence::prelude::EmployeeModel;
  use persistence::prelude::EmployeeRecord;
  use persistence::prelude::EmployeeRepository;
  use persistence::prelude::EmployeeRole;
  use persistence::prelude::EmployeeRuntimeConfig;
  use persistence::prelude::EmployeeStatus;
  use persistence::prelude::RunFilter;
  use persistence::prelude::RunModel;
  use persistence::prelude::RunRepository;
  use persistence::prelude::RunTrigger;
  use persistence::prelude::SurrealConnection;
  use tokio::time::sleep;
  use tokio::time::timeout;

  use super::*;

  static TEST_LOCK: Mutex<()> = Mutex::new(());
  static TEST_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::temp_dir().join(format!("blprnt-coordinator-tests-{}", std::process::id()));
    std::fs::create_dir_all(&path).expect("test home should exist");
    path
  });
  static TEST_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("failed to create test runtime")
  });

  struct CwdGuard {
    previous_cwd: PathBuf,
  }

  impl Drop for CwdGuard {
    fn drop(&mut self) {
      std::env::set_current_dir(&self.previous_cwd).expect("cwd should restore");
    }
  }

  fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  async fn prepare_environment() -> CwdGuard {
    unsafe { std::env::set_var("HOME", &*TEST_HOME) };
    let previous_cwd = std::env::current_dir().expect("cwd should resolve");
    std::env::set_current_dir(&*TEST_HOME).expect("cwd should switch");
    SurrealConnection::reset().await.expect("database should reset");

    CwdGuard { previous_cwd }
  }

  #[test]
  fn run_employee_once_returns_adapter_runtime_failures() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Runtime".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Runtime".to_string(),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let run =
        RunRepository::create(RunModel::new(employee.id, RunTrigger::Manual)).await.expect("run should be created");

      let coordinator = Coordinator::new();
      let runtime_state = Arc::new(EmployeeRuntimeState::new());
      let mut rx = COORDINATOR_EVENTS.subscribe();
      let expected_run_id = run.id.clone();

      let adapter_task = tokio::spawn(async move {
        loop {
          let event = rx.recv().await.expect("coordinator event should arrive");
            let CoordinatorEvent::StartRun { run_id, tx, .. } = event else {
              continue;
            };

          if run_id == expected_run_id {
            let sender = tx.lock().await.take().expect("adapter sender should be available");
            sender
              .send(Err(anyhow::anyhow!("adapter runtime failed")))
              .expect("coordinator should still be awaiting the adapter result");
            break;
          }
        }
      });

      let error = coordinator
        .run_employee_once(run, runtime_state, true)
        .await
        .expect_err("adapter runtime failures must propagate out of the coordinator");

      adapter_task.await.expect("adapter task should complete");
      assert!(error.to_string().contains("adapter runtime failed"));
    });
  }

  #[test]
  fn trigger_run_now_rejects_paused_employees_for_manual_triggers() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Paused Runtime".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Paused Runtime".to_string(),
        status: EmployeeStatus::Paused,
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          Arc::new(EmployeeRuntimeState::new()),
        },
      );

      let error = coordinator
        .trigger_run_now(&employee.id, None, RunTrigger::Manual)
        .await
        .expect_err("paused employees must reject manual runs");

      assert!(matches!(error, CoordinatorError::EmployeePaused));
      assert!(
        RunRepository::list(RunFilter { employee: Some(employee.id.clone()), issue: None, status: None, trigger: None })
        .await
        .expect("run list should load")
        .is_empty()
      );
    });
  }

  #[test]
  fn cancel_run_cancels_the_adapter_token_for_active_runs() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Cancelable Runtime".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Cancelable Runtime".to_string(),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let runtime_state = Arc::new(EmployeeRuntimeState::new());
      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          runtime_state.clone(),
        },
      );

      let mut rx = COORDINATOR_EVENTS.subscribe();
      let run = coordinator
        .trigger_run_now(&employee.id, None, RunTrigger::Manual)
        .await
        .expect("manual run should start")
        .expect("manual run should be created");

      let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("coordinator event should arrive")
        .expect("coordinator event should be readable");
      let CoordinatorEvent::StartRun { run_id, cancel_token, tx, .. } = event else {
        panic!("expected normal start run event");
      };
      assert_eq!(run_id, run.id);
      assert!(!cancel_token.is_cancelled(), "adapter token should start active");

      coordinator.cancel_run(&employee.id, &run.id).await;
      assert!(cancel_token.is_cancelled(), "cancelling the run should cancel the adapter token");

      let sender = tx.lock().await.take().expect("adapter sender should be available");
      let _ = sender.send(Err(anyhow::anyhow!("run cancelled")));
    });
  }

  #[test]
  fn upsert_employee_does_not_schedule_timer_runs_for_paused_employees() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Paused Scheduler".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Paused Scheduler".to_string(),
        status: EmployeeStatus::Paused,
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.upsert_employee(employee.id.clone(), SchedulerStartMode::Live).await;
      sleep(Duration::from_millis(100)).await;
      coordinator.remove_employee(&employee.id).await;

      assert!(
        RunRepository::list(RunFilter { employee: Some(employee.id.clone()), issue: None, status: None, trigger: None })
        .await
        .expect("run list should load")
        .is_empty()
      );
    });
  }

  #[test]
  fn trigger_run_now_allows_conversations_for_paused_employees_without_unpausing_them() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Paused Conversation".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Paused Conversation".to_string(),
        status: EmployeeStatus::Paused,
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          Arc::new(EmployeeRuntimeState::new()),
        },
      );

      let mut rx = COORDINATOR_EVENTS.subscribe();
      let run = coordinator
        .trigger_run_now(&employee.id, None, RunTrigger::Conversation)
        .await
        .expect("paused employees should allow conversation runs")
        .expect("conversation run should be created");

      let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("coordinator event should arrive")
        .expect("coordinator event should be readable");
      let CoordinatorEvent::StartRun { run_id, tx, .. } = event else {
        panic!("expected normal start run event");
      };
      assert_eq!(run_id, run.id);

      let sender = tx.lock().await.take().expect("adapter sender should be available");
      sender.send(Ok(())).expect("coordinator should still be awaiting the adapter result");
      sleep(Duration::from_millis(50)).await;

      let employee = EmployeeRepository::get(employee.id).await.expect("employee should load");
      assert_eq!(employee.status, EmployeeStatus::Paused);
    });
  }

  #[test]
  fn conversation_runs_do_not_consume_capacity_slots() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Conversation Capacity".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Conversation Capacity".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(false),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let runtime_state = Arc::new(EmployeeRuntimeState::new());
      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          runtime_state.clone(),
        },
      );

      let mut rx = COORDINATOR_EVENTS.subscribe();
      let run = coordinator
        .trigger_run_now(&employee.id, None, RunTrigger::Conversation)
        .await
        .expect("conversation runs should bypass capacity checks")
        .expect("conversation run should be created");

      let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("coordinator event should arrive")
        .expect("coordinator event should be readable");
      let CoordinatorEvent::StartRun { run_id, tx, .. } = event else {
        panic!("expected normal start run event");
      };
      assert_eq!(run_id, run.id);

      assert!(
        runtime_state.try_reserve_slot(1),
        "conversation runs should leave the employee's counted capacity available"
      );
      assert!(
        !runtime_state.try_reserve_slot(1),
        "reserving the only slot after the conversation starts should exhaust counted capacity"
      );

      let sender = tx.lock().await.take().expect("adapter sender should be available");
      sender.send(Ok(())).expect("coordinator should still be awaiting the adapter result");
      sleep(Duration::from_millis(50)).await;

      runtime_state.release_slot();
    });
  }

  #[test]
  fn trigger_run_now_skips_issue_mentions_for_non_wake_on_demand_employees() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Mention Skip".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Mention Skip".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         false,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(false),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          Arc::new(EmployeeRuntimeState::new()),
        },
      );

      let result = coordinator
        .trigger_run_now(
          &employee.id,
          None,
          RunTrigger::IssueMention {
            issue_id:   persistence::prelude::IssueId::from(Uuid::new_v4()),
            comment_id: persistence::prelude::IssueCommentId::from(Uuid::new_v4()),
          },
        )
        .await
        .expect("non-wake employees should be skipped cleanly");

      assert!(result.is_none());
      assert!(
        RunRepository::list(RunFilter { employee: Some(employee.id.clone()), issue: None, status: None, trigger: None })
        .await
        .expect("run list should load")
        .is_empty()
      );
    });
  }

  #[test]
  fn trigger_run_now_rejects_paused_employees_for_issue_mentions() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Paused Mention Runtime".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Paused Mention Runtime".to_string(),
        status: EmployeeStatus::Paused,
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(false),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          Arc::new(EmployeeRuntimeState::new()),
        },
      );

      let error = coordinator
        .trigger_run_now(
          &employee.id,
          None,
          RunTrigger::IssueMention {
            issue_id:   persistence::prelude::IssueId::from(Uuid::new_v4()),
            comment_id: persistence::prelude::IssueCommentId::from(Uuid::new_v4()),
          },
        )
        .await
        .expect_err("paused employees must reject mention-triggered runs");

      assert!(matches!(error, CoordinatorError::EmployeePaused));
    });
  }

  #[test]
  fn live_initial_sleep_waits_full_heartbeat_for_never_run_employees() {
    let employee = EmployeeRecord {
      id:              EmployeeId::from(Uuid::nil()),
      name:            "Never Run".to_string(),
      kind:            EmployeeKind::Agent,
      role:            EmployeeRole::Staff,
      title:           "Never Run".to_string(),
      status:          EmployeeStatus::Idle,
      icon:            String::new(),
      color:           String::new(),
      capabilities:    Vec::new(),
      permissions:     Default::default(),
      reports_to:      None,
      provider_config: None,
      runtime_config:  Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 120,
        heartbeat_prompt:       String::new(),
        wake_on_demand:         true,
        timer_wakeups_enabled:  Some(true),
        dreams_enabled:         Some(false),
        max_concurrent_runs:    1,
        skill_stack:            None,
        reasoning_effort:       None,
      }),
      created_at:      Utc::now(),
      last_run_at:     None,
      updated_at:      Utc::now(),
      reports:         Vec::new(),
    };

    assert_eq!(Coordinator::initial_sleep_duration(&employee, SchedulerStartMode::Live), Duration::from_secs(120));
  }

  #[test]
  fn bootstrap_initial_sleep_staggers_only_overdue_employees() {
    let now = Utc::now();
    let mut employee = EmployeeRecord {
      id:              EmployeeId::from(Uuid::nil()),
      name:            "Boot".to_string(),
      kind:            EmployeeKind::Agent,
      role:            EmployeeRole::Staff,
      title:           "Boot".to_string(),
      status:          EmployeeStatus::Idle,
      icon:            String::new(),
      color:           String::new(),
      capabilities:    Vec::new(),
      permissions:     Default::default(),
      reports_to:      None,
      provider_config: None,
      runtime_config:  Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 60,
        heartbeat_prompt:       String::new(),
        wake_on_demand:         true,
        timer_wakeups_enabled:  Some(true),
        dreams_enabled:         Some(false),
        max_concurrent_runs:    1,
        skill_stack:            None,
        reasoning_effort:       None,
      }),
      created_at:      now,
      last_run_at:     Some(now - chrono::Duration::seconds(120)),
      updated_at:      now,
      reports:         Vec::new(),
    };

    assert_eq!(
      Coordinator::initial_sleep_duration(&employee, SchedulerStartMode::Bootstrap { position: 3 }),
      Duration::from_millis(Coordinator::BOOT_RUN_STAGGER_MS * 3)
    );

    employee.last_run_at = Some(now - chrono::Duration::seconds(10));
    let remaining = Coordinator::initial_sleep_duration(&employee, SchedulerStartMode::Bootstrap { position: 3 });
    assert!(remaining > Duration::from_secs(45));
    assert!(remaining <= Duration::from_secs(60));
  }

  #[test]
  fn live_upsert_does_not_schedule_immediate_timer_runs_for_never_run_employees() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "New Scheduler".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "New Scheduler".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(false),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.upsert_employee(employee.id.clone(), SchedulerStartMode::Live).await;
      sleep(Duration::from_millis(100)).await;
      coordinator.remove_employee(&employee.id).await;

      assert!(
        RunRepository::list(RunFilter { employee: Some(employee.id.clone()), issue: None, status: None, trigger: None })
        .await
        .expect("run list should load")
        .is_empty()
      );
    });
  }

  #[test]
  fn upsert_employee_does_not_schedule_timer_runs_when_timer_wakeups_are_disabled() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Timer Disabled".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Timer Disabled".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 0,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(false),
          dreams_enabled:         Some(true),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.upsert_employee(employee.id.clone(), SchedulerStartMode::Live).await;
      sleep(Duration::from_millis(100)).await;
      coordinator.remove_employee(&employee.id).await;

      assert!(
        RunRepository::list(RunFilter { employee: Some(employee.id.clone()), issue: None, status: None, trigger: None })
        .await
        .expect("run list should load")
        .is_empty()
      );

      let employee = EmployeeRepository::get(employee.id).await.expect("employee should load");
      assert_eq!(employee.status, EmployeeStatus::Idle);
    });
  }

  #[test]
  fn init_fails_running_runs_for_employees_stuck_in_running_state() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Interrupted Runtime".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Interrupted Runtime".to_string(),
        status: EmployeeStatus::Running,
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let run = RunRepository::create(RunModel::new(employee.id.clone(), RunTrigger::Manual))
        .await
        .expect("run should be created");
      let run = RunRepository::update(run.id, RunStatus::Running).await.expect("run should be marked running");

      let coordinator = Coordinator::new();
      coordinator.init().await.expect("coordinator init should succeed");

      let employee = EmployeeRepository::get(employee.id).await.expect("employee should load");
      let run = RunRepository::get(run.id).await.expect("run should load");

      assert_eq!(employee.status, EmployeeStatus::Idle);
      assert!(matches!(run.status, RunStatus::Failed(reason) if reason == Coordinator::INTERRUPTED_RUN_REASON));
      assert!(run.completed_at.is_some());
    });
  }

  #[test]
  fn init_fails_running_runs_even_when_the_employee_is_not_marked_running() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Idle Runtime".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Idle Runtime".to_string(),
        status: EmployeeStatus::Idle,
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let run = RunRepository::create(RunModel::new(employee.id.clone(), RunTrigger::Timer))
        .await
        .expect("run should be created");
      let run = RunRepository::update(run.id, RunStatus::Running).await.expect("run should be marked running");

      let coordinator = Coordinator::new();
      coordinator.init().await.expect("coordinator init should succeed");

      let employee = EmployeeRepository::get(employee.id).await.expect("employee should load");
      let run = RunRepository::get(run.id).await.expect("run should load");

      assert_eq!(employee.status, EmployeeStatus::Idle);
      assert!(matches!(run.status, RunStatus::Failed(reason) if reason == Coordinator::INTERRUPTED_RUN_REASON));
      assert!(run.completed_at.is_some());
    });
  }

  #[test]
  fn run_employee_once_marks_runs_failed_when_the_adapter_handoff_fails() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Missing Listener".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Missing Listener".to_string(),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let run = RunRepository::create(RunModel::new(employee.id.clone(), RunTrigger::Timer))
        .await
        .expect("run should be created");

      let coordinator = Coordinator::new();
      let runtime_state = Arc::new(EmployeeRuntimeState::new());
      assert!(runtime_state.try_reserve_slot(1), "counted slot should be reserved before the run starts");

      let error = coordinator
        .run_employee_once(run.clone(), runtime_state, true)
        .await
        .expect_err("missing adapter listeners must fail the handoff");

      assert!(matches!(error, CoordinatorError::FailedToEmitCoordinatorEvent(_)));

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(
        matches!(run.status, RunStatus::Failed(ref reason) if reason.contains("failed to emit coordinator event")),
        "run should be marked failed when the adapter handoff fails: {:?}",
        run.status
      );
      assert!(run.completed_at.is_some());
    });
  }

  #[test]
  fn listen_processes_back_to_back_api_events_without_dropping_later_messages() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Event Ordering".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Event Ordering".to_string(),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      let listen_task = tokio::spawn({
        let coordinator = coordinator.clone();
        async move { coordinator.listen().await }
      });

      sleep(Duration::from_millis(50)).await;

      API_EVENTS
        .emit(ApiEvent::AddEmployee { employee_id: employee.id.clone() })
        .expect("add employee event should emit");
      API_EVENTS
        .emit(ApiEvent::DeleteEmployee { employee_id: employee.id.clone() })
        .expect("delete employee event should emit");

      for _ in 0..20 {
        if coordinator.schedules.read().await.is_empty() {
          break;
        }
        sleep(Duration::from_millis(25)).await;
      }

      assert!(
        coordinator.schedules.read().await.is_empty(),
        "coordinator should process both queued events without dropping the delete event"
      );

      listen_task.abort();
      let _ = listen_task.await;
    });
  }

  #[test]
  fn scheduler_skips_dreaming_runs_when_employee_has_dreams_disabled() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Dreams Disabled".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Dreams Disabled".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 0,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(false),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.upsert_employee(employee.id.clone(), SchedulerStartMode::Live).await;
      sleep(Duration::from_millis(100)).await;
      coordinator.remove_employee(&employee.id).await;

      let runs = RunRepository::list(RunFilter {
        employee: Some(employee.id),
        issue: None,
        status: None,
        trigger: None,
      })
      .await
      .expect("runs should load");
      assert!(runs.iter().all(|run| !matches!(run.trigger, RunTrigger::Dreaming)));
    });
  }

  #[test]
  fn dreaming_runs_bypass_capacity_slots_and_are_deleted_after_minion_handoff() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Minion Dreaming".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Minion Dreaming".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(true),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let runtime_state = Arc::new(EmployeeRuntimeState::new());
      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          runtime_state.clone(),
        },
      );

      let mut rx = COORDINATOR_EVENTS.subscribe();
      let run = coordinator
        .trigger_minion_dream_now(&employee.id)
        .await
        .expect("dreaming runs should bypass capacity checks")
        ;

      let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("coordinator event should arrive")
        .expect("coordinator event should be readable");

      let CoordinatorEvent::StartMinionRun { employee_id, kind, tx, .. } = event else {
        panic!("expected minion run event");
      };
      assert_eq!(employee_id, employee.id);
      assert!(matches!(kind, MinionRunKind::Dreamer));

      assert!(
        runtime_state.try_reserve_slot(1),
        "dreaming minion runs should leave the employee's counted capacity available"
      );

      let sender = tx.lock().await.take().expect("adapter sender should be available");
      sender.send(Ok(())).expect("coordinator should still be awaiting the adapter result");
      sleep(Duration::from_millis(50)).await;

      assert!(RunRepository::get(run.id.clone()).await.is_err(), "minion dreaming runs should not persist a database record");
      runtime_state.release_slot();
    });
  }

  #[test]
  fn regular_trigger_path_rejects_dreaming_trigger() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Regular Trigger".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Regular Trigger".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(true),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let coordinator = Coordinator::new();
      coordinator.schedules.write().await.insert(
        employee.id.clone(),
        EmployeeScheduleEntry {
          scheduler_cancel_token: CancellationToken::new(),
          runtime_state:          Arc::new(EmployeeRuntimeState::new()),
        },
      );

      let error = coordinator
        .trigger_run_now(&employee.id, None, RunTrigger::Dreaming)
        .await
        .expect_err("regular trigger path should reject dreaming");

      assert!(matches!(error, CoordinatorError::MinionOnlyTrigger));
    });
  }

  #[test]
  fn dreaming_stamp_blocks_same_day_scheduler_reentry_without_relying_on_persisted_runs() {
    let _lock = test_lock();

    TEST_RUNTIME.block_on(async {
      let _cwd = prepare_environment().await;
      let employee = EmployeeRepository::create(EmployeeModel {
        name: "Dream Stamp".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Dream Stamp".to_string(),
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 60,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(true),
          max_concurrent_runs:    1,
          skill_stack:            None,
          reasoning_effort:       None,
        }),
        ..Default::default()
      })
      .await
      .expect("employee should be created");

      let today = Utc::now().date_naive();
      let stamp_path = Coordinator::dreaming_stamp_path(&employee.id, today);
      fs::create_dir_all(stamp_path.parent().expect("stamp parent should exist")).expect("stamp dir should create");
      fs::write(&stamp_path, today.format("%Y-%m-%d").to_string()).expect("stamp should write");

      assert!(Coordinator::has_dreaming_run_for_today(&employee.id).await);

      let runs = RunRepository::list(RunFilter {
        employee: Some(employee.id),
        issue: None,
        status: None,
        trigger: None,
      })
      .await
      .expect("runs should load");
      assert!(runs.is_empty(), "same-day dream gating should not require persisted dreaming runs");
    });
  }
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use events::API_EVENTS;
use events::ApiEvent;
use events::COORDINATOR_EVENTS;
use events::CoordinatorEvent;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeStatus;
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

  pub fn new() -> Arc<Self> {
    Arc::new(Self { schedules: Arc::new(RwLock::new(HashMap::new())) })
  }

  pub async fn init(self: &Arc<Self>) -> CoordinatorResult<()> {
    RunRepository::mark_all_pending_as_failed("system shutdown".to_string())
      .await
      .map_err(CoordinatorError::DatabaseError)?;

    let mut employees = EmployeeRepository::list_agents().await.map_err(CoordinatorError::DatabaseError)?;

    employees.retain(|e| e.kind.is_agent());

    for employee in &employees {
      if employee.status == EmployeeStatus::Running {
        Self::mark_employee_idle(&employee.id).await?;
      }
    }

    for (position, employee) in employees.into_iter().enumerate() {
      self.upsert_employee(employee.id.clone(), SchedulerStartMode::Bootstrap { position }).await;
    }

    Ok(())
  }

  pub async fn listen(self: Arc<Self>) {
    loop {
      let event = API_EVENTS.subscribe().recv().await;

      match event {
        Ok(event) => match event {
          ApiEvent::StartRun { employee_id, trigger, rx } => {
            let run = self.trigger_run_now(&employee_id, trigger).await;
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
    run_trigger: RunTrigger,
  ) -> CoordinatorResult<Option<RunRecord>> {
    let employee = Self::load_employee(employee_id).await?;

    if employee.status == EmployeeStatus::Paused {
      tracing::info!("Employee {:?} is paused, skipping run", employee_id);
      return Err(CoordinatorError::EmployeePaused);
    }

    if matches!(run_trigger, RunTrigger::IssueAssignment { .. })
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

    if !runtime_state.try_reserve_slot(max_concurrent_runs) {
      return Err(CoordinatorError::NoRunSlotsAvailable);
    }

    if let Err(error) = Self::mark_employee_started(employee_id).await {
      tracing::error!(?employee_id, ?error, "failed to mark employee started {:?}", employee_id);
      let remaining_count = runtime_state.release_slot();

      if remaining_count == 0 {
        let _ = Self::mark_employee_idle(employee_id).await;
      }

      return Err(error);
    }

    let run = RunRepository::create(RunModel::new(employee_id.clone(), run_trigger))
      .await
      .map_err(CoordinatorError::DatabaseError)?;

    self.spawn_employee_run(run.clone(), runtime_state);

    Ok(Some(run))
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

      let Ok(run) = RunRepository::create(RunModel::new(employee_id.clone(), RunTrigger::Timer)).await else {
        tracing::error!(?employee_id, "failed to create run");
        continue;
      };

      let max_concurrent_runs =
        employee.runtime_config.as_ref().map(|config| config.max_concurrent_runs.max(1) as usize).unwrap_or(1);

      if !Self::wait_for_capacity(runtime_state.clone(), max_concurrent_runs, &scheduler_cancel_token).await {
        break;
      }

      if let Err(error) = Self::mark_employee_started(&employee_id).await {
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

      self.spawn_employee_run(run, run_runtime_state);
    }
  }

  fn spawn_employee_run(self: &Arc<Self>, run: RunRecord, runtime_state: Arc<EmployeeRuntimeState>) {
    let coordinator = self.clone();

    tokio::spawn(async move {
      let employee_id = run.employee_id.clone();
      let run_id = run.id.clone();
      let run_result = coordinator.run_employee_once(run, runtime_state.clone()).await;

      if let Err(error) = run_result {
        tracing::error!(?employee_id, ?error, "employee run failed");
      }

      let Some(remaining_count) = runtime_state.finish_run(&run_id).await else {
        tracing::info!("No remaining count for employee {:?}", employee_id);
        return;
      };

      if remaining_count == 0
        && let Err(error) = Self::mark_employee_idle(&employee_id).await
      {
        tracing::error!(?employee_id, ?error, "failed to mark employee idle");
      }
    });
  }

  async fn run_employee_once(&self, run: RunRecord, runtime_state: Arc<EmployeeRuntimeState>) -> CoordinatorResult<()> {
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<()>>();

    let run_id = run.id.clone();
    let run_cancel_token = CancellationToken::new();
    runtime_state.register_run(run_id.clone(), run_cancel_token.child_token()).await;

    let _ = RunRepository::update(run_id.clone(), RunStatus::Running).await.map_err(CoordinatorError::DatabaseError)?;

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

  async fn load_employee(employee_id: &EmployeeId) -> CoordinatorResult<EmployeeRecord> {
    Ok(EmployeeRepository::get(employee_id.clone()).await.map_err(CoordinatorError::DatabaseError)?)
  }

  async fn mark_employee_started(employee_id: &EmployeeId) -> CoordinatorResult<()> {
    EmployeeRepository::update(
      employee_id.clone(),
      EmployeePatch { status: Some(EmployeeStatus::Running), last_run_at: Some(Utc::now()), ..Default::default() },
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
          let CoordinatorEvent::StartRun { run_id, tx, .. } = event;

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
        .run_employee_once(run, runtime_state)
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
        .trigger_run_now(&employee.id, RunTrigger::Manual)
        .await
        .expect_err("paused employees must reject manual runs");

      assert!(matches!(error, CoordinatorError::EmployeePaused));
      assert!(
        RunRepository::list(RunFilter { employee: None, status: None, trigger: None })
          .await
          .expect("run list should load")
          .is_empty()
      );
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
        RunRepository::list(RunFilter { employee: None, status: None, trigger: None })
          .await
          .expect("run list should load")
          .is_empty()
      );
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
        max_concurrent_runs:    1,
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
        max_concurrent_runs:    1,
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
          max_concurrent_runs:    1,
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
        RunRepository::list(RunFilter { employee: None, status: None, trigger: None })
          .await
          .expect("run list should load")
          .is_empty()
      );
    });
  }
}

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use chrono::Utc;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeStatus;
use tokio::sync::Notify;
use tokio::sync::OnceCell;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

static RUN_MANAGER: OnceCell<Arc<RunManager>> = OnceCell::const_new();

struct EmployeeRuntimeState {
  running_count:     AtomicUsize,
  completion_notify: Notify,
}

impl EmployeeRuntimeState {
  fn new() -> Self {
    Self { running_count: AtomicUsize::new(0), completion_notify: Notify::new() }
  }

  fn try_reserve_slot(&self, max_concurrent_runs: usize) -> bool {
    loop {
      let running_count = self.running_count.load(Ordering::Acquire);

      if running_count >= max_concurrent_runs {
        return false;
      }

      if self
        .running_count
        .compare_exchange(running_count, running_count + 1, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
      {
        return true;
      }
    }
  }

  fn release_slot(&self) -> usize {
    let previous_count = self.running_count.fetch_sub(1, Ordering::AcqRel);
    let remaining_count = previous_count.saturating_sub(1);

    self.completion_notify.notify_one();

    remaining_count
  }
}

struct EmployeeScheduleEntry {
  cancellation_token: CancellationToken,
  runtime_state:      Arc<EmployeeRuntimeState>,
}

pub struct RunManager {
  schedules: Arc<RwLock<HashMap<EmployeeId, EmployeeScheduleEntry>>>,
}

impl RunManager {
  pub async fn get() -> Arc<Self> {
    RUN_MANAGER
      .get_or_init(|| async {
        let manager = Arc::new(Self::new());
        manager.init().await.expect("failed to initialize run manager");
        manager
      })
      .await
      .clone()
  }

  fn new() -> Self {
    Self { schedules: Arc::new(RwLock::new(HashMap::new())) }
  }

  async fn init(self: &Arc<Self>) -> Result<()> {
    let employees = EmployeeRepository::list_agents().await?;

    for employee in &employees {
      if employee.status == EmployeeStatus::Running {
        Self::mark_employee_idle(&employee.id).await?;
      }
    }

    for employee in employees {
      self.upsert_employee(employee.id.clone()).await;
    }

    Ok(())
  }

  pub async fn upsert_employee(self: &Arc<Self>, employee_id: EmployeeId) {
    let runtime_state = {
      let mut schedules = self.schedules.write().await;

      match schedules.remove(&employee_id) {
        Some(existing) => {
          existing.cancellation_token.cancel();
          existing.runtime_state
        }
        None => Arc::new(EmployeeRuntimeState::new()),
      }
    };

    if !Self::load_employee(&employee_id).await.is_err() {
      let cancellation_token = CancellationToken::new();

      self.schedules.write().await.insert(
        employee_id.clone(),
        EmployeeScheduleEntry {
          cancellation_token: cancellation_token.clone(),
          runtime_state:      runtime_state.clone(),
        },
      );

      tokio::spawn(self.clone().employee_scheduler_loop(employee_id, runtime_state, cancellation_token));
    }
  }

  pub async fn trigger_run_now(self: &Arc<Self>, employee_id: &EmployeeId) -> Result<()> {
    let runtime_state = {
      let schedules = self.schedules.read().await;

      schedules
        .get(employee_id)
        .map(|entry| entry.runtime_state.clone())
        .ok_or_else(|| anyhow!("employee is not managed by RunManager"))?
    };

    let employee = Self::load_employee(employee_id).await?;

    let max_concurrent_runs =
      employee.runtime_config.as_ref().map(|config| config.max_concurrent_runs.max(1) as usize).unwrap_or(1);

    if !runtime_state.try_reserve_slot(max_concurrent_runs) {
      bail!("no run slots available for employee");
    }

    if let Err(error) = Self::mark_employee_started(employee_id).await {
      let remaining_count = runtime_state.release_slot();

      if remaining_count == 0 {
        let _ = Self::mark_employee_idle(employee_id).await;
      }

      return Err(error);
    }

    self.spawn_employee_run(employee_id.clone(), runtime_state);

    Ok(())
  }

  pub async fn remove_employee(&self, employee_id: &EmployeeId) {
    if let Some(existing) = self.schedules.write().await.remove(employee_id) {
      existing.cancellation_token.cancel();
    }
  }

  async fn employee_scheduler_loop(
    self: Arc<Self>,
    employee_id: EmployeeId,
    runtime_state: Arc<EmployeeRuntimeState>,
    cancellation_token: CancellationToken,
  ) {
    loop {
      let employee = match Self::load_employee(&employee_id).await {
        Ok(employee) => employee,
        Err(error) => {
          tracing::error!(?employee_id, ?error, "failed to load employee");
          if !wait_or_cancel(Duration::from_secs(30), &cancellation_token).await {
            break;
          }
          continue;
        }
      };

      let sleep_duration = next_due_duration(&employee);

      if !wait_or_cancel(sleep_duration, &cancellation_token).await {
        break;
      }

      let employee = match Self::load_employee(&employee_id).await {
        Ok(employee) => employee,
        Err(error) => {
          tracing::error!(?employee_id, ?error, "failed to reload employee");
          if !wait_or_cancel(Duration::from_secs(30), &cancellation_token).await {
            break;
          }
          continue;
        }
      };

      let max_concurrent_runs =
        employee.runtime_config.as_ref().map(|config| config.max_concurrent_runs.max(1) as usize).unwrap_or(1);

      if !wait_for_capacity(runtime_state.clone(), max_concurrent_runs, &cancellation_token).await {
        break;
      }

      if let Err(error) = Self::mark_employee_started(&employee_id).await {
        let remaining_count = runtime_state.release_slot();

        if remaining_count == 0 {
          let _ = Self::mark_employee_idle(&employee_id).await;
        }

        tracing::error!(?employee_id, ?error, "failed to mark employee started");

        if !wait_or_cancel(Duration::from_secs(10), &cancellation_token).await {
          break;
        }

        continue;
      }

      let run_employee_id = employee_id.clone();
      let run_runtime_state = runtime_state.clone();

      self.spawn_employee_run(run_employee_id, run_runtime_state);
    }
  }

  fn spawn_employee_run(self: &Arc<Self>, employee_id: EmployeeId, runtime_state: Arc<EmployeeRuntimeState>) {
    let manager = self.clone();

    tokio::spawn(async move {
      let run_result = manager.run_employee_once(employee_id.clone()).await;

      if let Err(error) = run_result {
        tracing::error!(?employee_id, ?error, "employee run failed");
      }

      let remaining_count = runtime_state.release_slot();

      if remaining_count == 0
        && let Err(error) = Self::mark_employee_idle(&employee_id).await
      {
        tracing::error!(?employee_id, ?error, "failed to mark employee idle");
      }
    });
  }

  async fn run_employee_once(&self, employee_id: EmployeeId) -> Result<()> {
    // Do the actual work here.
    let _ = employee_id;
    Ok(())
  }

  async fn load_employee(employee_id: &EmployeeId) -> Result<EmployeeRecord> {
    Ok(EmployeeRepository::get(employee_id.clone()).await?)
  }

  async fn mark_employee_started(employee_id: &EmployeeId) -> Result<()> {
    EmployeeRepository::update(
      employee_id.clone(),
      EmployeePatch { status: Some(EmployeeStatus::Running), last_run_at: Some(Utc::now()), ..Default::default() },
    )
    .await?;

    Ok(())
  }

  async fn mark_employee_idle(employee_id: &EmployeeId) -> Result<()> {
    EmployeeRepository::update(
      employee_id.clone(),
      EmployeePatch { status: Some(EmployeeStatus::Idle), ..Default::default() },
    )
    .await?;

    Ok(())
  }
}

fn next_due_duration(employee: &EmployeeRecord) -> Duration {
  let heartbeat_interval_sec =
    employee.runtime_config.as_ref().map(|config| config.heartbeat_interval_sec.max(0)).unwrap_or(3600);

  let Some(last_run_at) = employee.last_run_at else {
    return Duration::ZERO;
  };

  let next_run_at = last_run_at + chrono::Duration::seconds(heartbeat_interval_sec);
  let now = Utc::now();

  if next_run_at <= now { Duration::ZERO } else { (next_run_at - now).to_std().unwrap_or(Duration::ZERO) }
}

async fn wait_or_cancel(duration: Duration, cancellation_token: &CancellationToken) -> bool {
  tokio::select! {
    _ = tokio::time::sleep(duration) => true,
    _ = cancellation_token.cancelled() => false,
  }
}

async fn wait_for_capacity(
  runtime_state: Arc<EmployeeRuntimeState>,
  max_concurrent_runs: usize,
  cancellation_token: &CancellationToken,
) -> bool {
  loop {
    if runtime_state.try_reserve_slot(max_concurrent_runs) {
      return true;
    }

    tokio::select! {
      _ = runtime_state.completion_notify.notified() => {}
      _ = cancellation_token.cancelled() => return false,
    }
  }
}

use std::collections::HashMap;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use url::Url;

use crate::preview::detect::DevCommand;

const URL_CHANNEL_BUFFER: usize = 32;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub trait ProcessHandle: Send {
  fn start_kill(&mut self);
}

pub trait ProcessSpawner: Send + Sync {
  fn spawn(&self, working_dir: PathBuf, command: DevCommand) -> Result<SpawnedProcessHandle>;
}

pub struct SpawnedProcessHandle {
  pub pid:    Option<u32>,
  pub stdout: Option<Box<dyn AsyncRead + Unpin + Send>>,
  pub stderr: Option<Box<dyn AsyncRead + Unpin + Send>>,
  pub handle: Box<dyn ProcessHandle>,
}

#[derive(Debug)]
pub struct SpawnedProcess {
  pub pid:          Option<u32>,
  pub url_receiver: mpsc::Receiver<String>,
}

struct ManagedProcess {
  pid:         Option<u32>,
  handle:      Box<dyn ProcessHandle>,
  stdout_task: Option<JoinHandle<()>>,
  stderr_task: Option<JoinHandle<()>>,
}

#[derive(Debug)]
struct TokioProcessSpawner;

#[derive(Debug)]
struct TokioChildHandle {
  child: Child,
}

impl ProcessHandle for TokioChildHandle {
  fn start_kill(&mut self) {
    let _ = self.child.start_kill();
  }
}

impl ProcessSpawner for TokioProcessSpawner {
  fn spawn(&self, working_dir: PathBuf, command: DevCommand) -> Result<SpawnedProcessHandle> {
    let mut cmd = Command::new(&command.program);
    cmd.args(command.args);
    cmd.envs(command.env);
    cmd.current_dir(working_dir);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd.spawn()?;
    let pid = child.id();
    let stdout = child.stdout.take().map(|value| Box::new(value) as Box<dyn AsyncRead + Unpin + Send>);
    let stderr = child.stderr.take().map(|value| Box::new(value) as Box<dyn AsyncRead + Unpin + Send>);
    let handle = Box::new(TokioChildHandle { child });

    Ok(SpawnedProcessHandle { pid, stdout, stderr, handle })
  }
}

pub struct ProcessManager {
  processes: HashMap<String, ManagedProcess>,
  spawner:   Arc<dyn ProcessSpawner>,
}

impl ProcessManager {
  pub fn new() -> Self {
    Self { processes: HashMap::new(), spawner: Arc::new(TokioProcessSpawner) }
  }

  pub fn with_spawner(spawner: Arc<dyn ProcessSpawner>) -> Self {
    Self { processes: HashMap::new(), spawner }
  }

  pub fn spawn(&mut self, key: String, working_dir: PathBuf, command: DevCommand) -> Result<SpawnedProcess> {
    if self.processes.contains_key(&key) {
      self.stop(&key);
    }

    let SpawnedProcessHandle { pid, stdout, stderr, handle } = self.spawner.spawn(working_dir, command)?;
    let (url_sender, url_receiver) = mpsc::channel(URL_CHANNEL_BUFFER);

    let stdout_task = stdout.map(|stdout| tokio::spawn(read_output(stdout, url_sender.clone())));
    let stderr_task = stderr.map(|stderr| tokio::spawn(read_output(stderr, url_sender)));

    self.processes.insert(key, ManagedProcess { pid, handle, stdout_task, stderr_task });

    Ok(SpawnedProcess { pid, url_receiver })
  }

  pub fn stop(&mut self, key: &str) {
    let Some(mut process) = self.processes.remove(key) else {
      return;
    };

    process.handle.start_kill();
    if let Some(task) = process.stdout_task {
      task.abort();
    }
    if let Some(task) = process.stderr_task {
      task.abort();
    }
  }

  pub fn pid(&self, key: &str) -> Option<u32> {
    self.processes.get(key).and_then(|process| process.pid)
  }
}

async fn read_output<R: AsyncRead + Unpin + Send + 'static>(reader: R, sender: mpsc::Sender<String>) {
  let mut lines = BufReader::new(reader).lines();

  loop {
    let line = match lines.next_line().await {
      Ok(Some(line)) => line,
      _ => break,
    };

    if let Some(url) = parse_output_url(&line) {
      let _ = sender.send(url).await;
    }
  }
}

fn parse_output_url(line: &str) -> Option<String> {
  if let Some(url) = extract_url(line) {
    return Some(url);
  }

  extract_port(line).map(|port| format!("http://localhost:{}", port))
}

fn extract_url(line: &str) -> Option<String> {
  for token in line.split_whitespace() {
    let candidate =
      token.trim_matches(|ch: char| ch == ',' || ch == ';' || ch == '"' || ch == '\'' || ch == '(' || ch == ')');

    if (candidate.contains("http://") || candidate.contains("https://"))
      && let Some(url) = normalize_output_url(candidate)
    {
      return Some(url);
    }

    if candidate.starts_with("localhost:") || candidate.starts_with("127.0.0.1:") || candidate.starts_with("0.0.0.0:") {
      let candidate = format!("http://{}", candidate);
      if let Some(url) = normalize_output_url(&candidate) {
        return Some(url);
      }
    }
  }

  None
}

fn normalize_output_url(value: &str) -> Option<String> {
  let mut url = Url::parse(value).ok()?;
  url.set_fragment(None);
  url.set_query(None);
  let normalized = url.to_string();
  Some(normalized.trim_end_matches('/').to_string())
}

fn extract_port(line: &str) -> Option<u16> {
  let lower = line.to_lowercase();

  for token in lower.split_whitespace() {
    if let Some(port) = token.strip_prefix("localhost:").and_then(parse_port_value) {
      return Some(port);
    }
    if let Some(port) = token.strip_prefix("127.0.0.1:").and_then(parse_port_value) {
      return Some(port);
    }
    if let Some(port) = token.strip_prefix("0.0.0.0:").and_then(parse_port_value) {
      return Some(port);
    }
  }

  if let Some(port) = extract_port_after_keyword(&lower, "port") {
    return Some(port);
  }

  None
}

fn extract_port_after_keyword(text: &str, keyword: &str) -> Option<u16> {
  let index = text.find(keyword)?;
  let remainder = &text[index + keyword.len()..];
  let mut digits = String::new();
  let mut started = false;

  for ch in remainder.chars().take(50) {
    if ch.is_ascii_digit() {
      digits.push(ch);
      started = true;
      continue;
    }
    if started {
      break;
    }
  }

  if digits.is_empty() {
    return None;
  }

  digits.parse::<u16>().ok()
}

fn parse_port_value(value: &str) -> Option<u16> {
  let value = value.trim_matches(|ch: char| ch == ',' || ch == ';' || ch == '"' || ch == '\'' || ch == ')');
  let mut digits = String::new();

  for ch in value.chars() {
    if ch.is_ascii_digit() {
      digits.push(ch);
    } else if !digits.is_empty() {
      break;
    }
  }

  if digits.is_empty() {
    return None;
  }

  digits.parse::<u16>().ok()
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::sync::atomic::AtomicBool;
  use std::sync::atomic::Ordering;
  use std::time::Duration;

  use tokio::io::AsyncWriteExt;

  use super::*;

  struct FakeHandle {
    killed: Arc<AtomicBool>,
  }

  impl ProcessHandle for FakeHandle {
    fn start_kill(&mut self) {
      self.killed.store(true, Ordering::SeqCst);
    }
  }

  struct FakeSpawner {
    killed: Arc<AtomicBool>,
  }

  impl ProcessSpawner for FakeSpawner {
    fn spawn(&self, _working_dir: PathBuf, _command: DevCommand) -> Result<SpawnedProcessHandle> {
      let (stdout, mut stdout_writer) = tokio::io::duplex(128);
      let (stderr, mut stderr_writer) = tokio::io::duplex(128);
      let killed = self.killed.clone();

      tokio::spawn(async move {
        let _ = stdout_writer.write_all(b"Server running at http://localhost:3001\n").await;
      });

      tokio::spawn(async move {
        let _ = stderr_writer.write_all(b"Listening on 0.0.0.0:4002\n").await;
      });

      Ok(SpawnedProcessHandle {
        pid:    Some(4242),
        stdout: Some(Box::new(stdout)),
        stderr: Some(Box::new(stderr)),
        handle: Box::new(FakeHandle { killed }),
      })
    }
  }

  #[tokio::test]
  async fn process_manager_spawn_and_stop() {
    let killed = Arc::new(AtomicBool::new(false));
    let spawner = Arc::new(FakeSpawner { killed: killed.clone() });
    let mut manager = ProcessManager::with_spawner(spawner);

    let command =
      DevCommand { program: "fake".to_string(), args: vec!["serve".to_string()], env: HashMap::new() };
    let spawned = manager.spawn("test".to_string(), PathBuf::from("."), command).expect("spawned");

    assert_eq!(spawned.pid, Some(4242));
    assert_eq!(manager.pid("test"), Some(4242));

    let mut receiver = spawned.url_receiver;
    let first = tokio::time::timeout(Duration::from_millis(200), receiver.recv()).await.expect("receive url");
    assert!(first.is_some());

    manager.stop("test");

    assert!(killed.load(Ordering::SeqCst));
  }

  #[test]
  fn parse_output_url_prefers_url() {
    let line = "Local: http://localhost:3005/";
    assert_eq!(parse_output_url(line), Some("http://localhost:3005".to_string()));
  }

  #[test]
  fn parse_output_url_falls_back_to_port() {
    let line = "Server listening on port 4555";
    assert_eq!(parse_output_url(line), Some("http://localhost:4555".to_string()));
  }
}

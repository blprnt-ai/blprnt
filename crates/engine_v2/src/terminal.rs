#![allow(clippy::field_reassign_with_default)]

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

use common::tools::TerminalSnapshot;
use terminal::event::Event;
use terminal::event::EventListener;
use terminal::event::WindowSize;
use terminal::event_loop::EventLoop;
use terminal::event_loop::EventLoopSendError;
use terminal::event_loop::EventLoopSender;
use terminal::event_loop::Msg;
use terminal::grid::Dimensions;
use terminal::sync::FairMutex;
use terminal::term::Config as TermConfig;
use terminal::term::Term;
use terminal::tty::Options as PtyOptions;
use terminal::tty::{self};
use tokio::sync::watch;

#[derive(Clone)]
struct ChannelEventListener {
  sender: mpsc::Sender<Event>,
}

impl EventListener for ChannelEventListener {
  fn send_event(&self, event: Event) {
    let _ = self.sender.send(event);
  }
}

#[derive(Clone, Copy)]
struct TerminalDimensions {
  columns:      usize,
  screen_lines: usize,
  total_lines:  usize,
}

impl Dimensions for TerminalDimensions {
  fn total_lines(&self) -> usize {
    self.total_lines
  }

  fn screen_lines(&self) -> usize {
    self.screen_lines
  }

  fn columns(&self) -> usize {
    self.columns
  }
}

pub struct TerminalManager {
  msg_sender:  Arc<EventLoopSender>,
  snapshot_rx: watch::Receiver<TerminalSnapshot>,
}

impl TerminalManager {
  pub fn spawn(working_directory: PathBuf) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let initial_size = WindowSize { num_cols: 120, num_lines: 10, cell_width: 8, cell_height: 16 };
    let dimensions = TerminalDimensions {
      columns:      initial_size.num_cols as usize,
      screen_lines: initial_size.num_lines as usize,
      total_lines:  initial_size.num_lines as usize,
    };

    // Alacritty core event channel (std mpsc).
    let (core_event_tx, core_event_rx) = mpsc::channel::<Event>();
    let event_listener = ChannelEventListener { sender: core_event_tx };

    // Terminal state.
    let term_config = TermConfig::default();
    let term = Term::new(term_config, &dimensions, event_listener.clone());
    let term = Arc::new(FairMutex::new(term));

    // PTY + child (shell).
    let mut pty_options = PtyOptions::default();
    pty_options.working_directory = Some(working_directory);
    let window_id: u64 = 1;
    let pty = tty::new(&pty_options, initial_size, window_id)?;

    // EventLoop (reads PTY -> updates Term, accepts Msg::Input/Resize/Shutdown).
    let drain_on_exit = pty_options.drain_on_exit;
    let ref_test = false;
    let event_loop = EventLoop::new(term.clone(), event_listener.clone(), pty, drain_on_exit, ref_test)?;

    let msg_sender = event_loop.channel();
    let _join = event_loop.spawn();

    // watch channel for “latest snapshot”
    let initial_snapshot = TerminalSnapshot {
      rows:  dimensions.screen_lines,
      cols:  dimensions.columns,
      lines: vec![String::new(); dimensions.screen_lines],
    };
    let (snapshot_tx, snapshot_rx) = watch::channel(initial_snapshot);

    // Dispatcher thread: on Wakeup, capture snapshot from Term and publish.
    std::thread::spawn({
      let msg_sender = msg_sender.clone();
      move || {
        for event in core_event_rx.iter() {
          match event {
            Event::Wakeup => {
              let snapshot = capture_text_snapshot(&term, &dimensions);
              let _ = snapshot_tx.send(snapshot);
            }
            Event::Exit | Event::ChildExit(_) => {
              // Publish one last snapshot if you want; then stop.
              let snapshot = capture_text_snapshot(&term, &dimensions);
              let _ = snapshot_tx.send(snapshot);
              break;
            }
            _ => {}
          }
        }

        let _ = msg_sender.send(Msg::Shutdown);
      }
    });

    Ok(Self { msg_sender, snapshot_rx })
  }

  pub fn write(&self, bytes: impl Into<Vec<u8>>) -> Result<(), EventLoopSendError> {
    // std::mpsc::Sender is sync; wrap in spawn_blocking to avoid blocking a Tokio core thread.
    let sender = self.msg_sender.clone();
    let data = bytes.into();
    sender.send(Msg::Input(Cow::Owned(data)))
  }

  pub fn close(&self) -> Result<(), EventLoopSendError> {
    let sender = self.msg_sender.clone();
    sender.send(Msg::Shutdown)
  }

  pub fn snapshot_text(&self) -> TerminalSnapshot {
    self.snapshot_rx.borrow().clone()
  }

  // If you want polling/waiting semantics:
  pub async fn wait_for_snapshot_change(&mut self) -> Result<TerminalSnapshot, watch::error::RecvError> {
    self.snapshot_rx.changed().await?;
    Ok(self.snapshot_rx.borrow().clone())
  }
}

type TerminalTerm = Term<ChannelEventListener>;

fn capture_text_snapshot(term: &Arc<FairMutex<TerminalTerm>>, dimensions: &TerminalDimensions) -> TerminalSnapshot {
  let mut term_guard = term.lock();
  let renderable = term_guard.renderable_content();

  let rows = dimensions.screen_lines;
  let cols = dimensions.columns;
  let mut lines = vec![String::with_capacity(cols); rows];

  for indexed_cell in renderable.display_iter {
    let row_index = indexed_cell.point.line.0 as usize;
    let col_index = indexed_cell.point.column.0;

    if row_index >= rows || col_index >= cols {
      continue;
    }

    while lines[row_index].chars().count() < col_index {
      lines[row_index].push(' ');
    }
    lines[row_index].push(indexed_cell.cell.c);
  }

  let _ = term_guard.damage();
  term_guard.reset_damage();

  TerminalSnapshot { rows, cols, lines }
}

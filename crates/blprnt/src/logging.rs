use std::fmt;

use colored::Colorize;
use tracing::Event;
use tracing::Subscriber;
use tracing::field::Field;
use tracing::field::Visit;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

struct Formatter;

impl<S, N> FormatEvent<S, N> for Formatter
where
  S: Subscriber + for<'lookup> LookupSpan<'lookup>,
  N: for<'writer> FormatFields<'writer> + 'static,
{
  fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> fmt::Result {
    let metadata = event.metadata();

    let level = match *metadata.level() {
      tracing::Level::ERROR => "ERROR".truecolor(208, 0, 0).to_string(),
      tracing::Level::WARN => "WARN!".truecolor(208, 85, 0).to_string(),
      tracing::Level::INFO => ">INFO".truecolor(208, 208, 0).to_string(),
      tracing::Level::DEBUG => "DEBUG".truecolor(0, 208, 0).to_string(),
      tracing::Level::TRACE => "TRACE".truecolor(0, 85, 208).to_string(),
    };

    let timestamp = chrono::Local::now().format("%I:%M:%S%.6f").to_string().truecolor(25, 100, 25);

    let target = metadata.target().to_string().truecolor(125, 125, 125);

    let file = metadata.file().map(|f| {
      match metadata.line() {
        Some(line) => format!("{f}:{line}"),
        None => f.to_string(),
      }
      .truecolor(125, 125, 125)
    });

    let middle = "├─".truecolor(125, 125, 125);
    let last = "└─".truecolor(125, 125, 125);

    #[derive(Default)]
    struct Visitor {
      message: String,
    }

    impl Visit for Visitor {
      fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
          self.message = value.to_string();
        }
      }

      fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
          let mut s = format!("{value:?}");
          if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            s = s[1..s.len() - 1].to_string();
          }
          self.message = s;
        }
      }
    }

    let mut visitor = Visitor::default();
    event.record(&mut visitor);

    let message = visitor
      .message
      .lines()
      .enumerate()
      .map(|(i, line)| if i == 0 { line.to_string() } else { format!("       {line}") })
      .collect::<Vec<_>>()
      .join("\n")
      .truecolor(200, 200, 200);

    if let Some(file) = file {
      writeln!(writer, "{level}  {timestamp}\n    {middle} {target}\n    {middle} {file}\n    {last} {message}")?;
    } else {
      writeln!(writer, "{level}  {timestamp}\n    {middle} {target}\n    {last} {message}")?;
    }

    for span in ctx.event_scope().into_iter().flat_map(|s| s.from_root()) {
      writeln!(writer, "       in {}", span.name().truecolor(90, 90, 90))?;
    }

    Ok(())
  }
}

pub fn init_logging() {
  let filters = vec![
    "warn",
    "adapters=info",
    "api=info",
    "blprnt=info",
    "coordinator=info",
    "events=info",
    "json-repair=info",
    "oauth=info",
    "persistence=info",
    "sandbox=info",
    "terminal=info",
    "tools=info",
    "vault=info",
  ]
  .join(",");

  tracing_subscriber::registry()
    .with(EnvFilter::new(filters))
    .with(tracing_subscriber::fmt::layer().event_format(Formatter))
    .init();
}

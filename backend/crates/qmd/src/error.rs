use thiserror::Error;

#[derive(Debug, Error)]
pub enum QmdError {
  #[error("not implemented: {op}")]
  NotImplemented { op: &'static str },

  #[error("invalid argument: {message}")]
  InvalidArgument { message: String },

  #[error("storage error: {message}")]
  Storage { message: String },

  #[error("llm error: {message}")]
  Llm { message: String },
}

pub type Result<T> = std::result::Result<T, QmdError>;

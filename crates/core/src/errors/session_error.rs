use thiserror::Error;

use crate::errors::pty_error::PtyError;

#[derive(Debug, Error)]
pub enum SessionError {
  #[error(transparent)]
  Pty(#[from] PtyError),
  #[error("session is closed")]
  Closed,
  #[error("model thread stopped")]
  ModelStopped,
  #[error("model query timed out")]
  Timeout,
  #[error("model query failed: {0}")]
  Query(String),
}

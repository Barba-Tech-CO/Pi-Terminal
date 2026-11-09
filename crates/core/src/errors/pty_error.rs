use thiserror::Error;

#[derive(Debug, Error)]
pub enum PtyError {
  #[error("failed to open pty: {0}")]
  Open(String),
  #[error("failed to spawn command: {0}")]
  Spawn(String),
  #[error("failed to obtain pty reader: {0}")]
  Reader(String),
  #[error("failed to obtain pty writer: {0}")]
  Writer(String),
  #[error("pty reader was already taken")]
  ReaderTaken,
  #[error("failed to read from pty: {0}")]
  Read(String),
  #[error("pty writer was already taken")]
  WriterTaken,
  #[error("failed to write to pty: {0}")]
  Write(String),
  #[error("failed to resize pty: {0}")]
  Resize(String),
  #[error("failed to query pty size: {0}")]
  Size(String),
  #[error("failed to wait for child: {0}")]
  Wait(String),
  #[error("failed to kill child: {0}")]
  Kill(String),
}

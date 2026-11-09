use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
  #[error("failed to compute semantic zones: {0}")]
  SemanticZones(String),
}

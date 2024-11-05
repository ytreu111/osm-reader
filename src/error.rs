use prost::DecodeError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, OsmPbfError>;

#[derive(Debug, Error)]
pub enum OsmPbfError {
  #[error("Error read size")]
  ReadHeaderSizeError,
  #[error("")]
  DecodeBlobHeaderError,
  #[error("")]
  DecodeBlobError(DecodeError),
  #[error("")]
  DecodeBlobDataError,
  #[error("")]
  UnsupportedCompressedType,
  #[error("")]
  DecodeError(DecodeError),
  #[error("TODO ERROR")]
  TodoError,
}

impl From<DecodeError> for OsmPbfError {
  fn from(e: DecodeError) -> Self {
    Self::DecodeError(e)
  }
}
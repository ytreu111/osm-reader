use prost::DecodeError;
use thiserror::Error;

pub fn new_blob_error(e: BlobError) -> OsmReaderError {
  OsmReaderError(Box::new(ErrorKind::Blob(e)))
}

pub fn new_proto_error(err: DecodeError) -> OsmReaderError {
  OsmReaderError(Box::new(ErrorKind::ProtoError(err)))
}

pub type Result<T> = std::result::Result<T, OsmPbfError>;
pub type OsmResult<T> = std::result::Result<T, OsmReaderError>;

#[derive(Debug)]
pub struct OsmReaderError(Box<ErrorKind>);

#[derive(Debug)]
pub enum ErrorKind {
  Blob(BlobError),
  ProtoError(DecodeError),
}

impl From<DecodeError> for OsmReaderError {
  fn from(e: DecodeError) -> OsmReaderError {
    OsmReaderError(Box::new(ErrorKind::ProtoError(e)))
  }
}

#[derive(Debug)]
pub enum BlobError {
  HeaderTooLarge(u32),
  ReadHeaderSizeError,
  DecodeBlobHeaderError,
  UnsupportedCompressedType,
  DecodeBlobDataError
}


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

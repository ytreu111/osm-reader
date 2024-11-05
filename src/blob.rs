use std::io::{ErrorKind, Read};
use bytes::{Buf, Bytes, BytesMut};
use flate2::read::ZlibDecoder;
use fmmap::{MmapFileExt};
use prost::Message;
use byteorder::ReadBytesExt;
use crate::block::{HeaderBlock, PrimitiveBlock};
use crate::proto;
use crate::error;
use crate::error::OsmPbfError;

const MAX_BLOB_HEADER_SIZE: u32 = 64 * 1024;
const MAX_BLOB_MESSAGE_SIZE: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlobType<'a> {
  OsmHeader,
  OsmData,
  Unknown(&'a str),
}

impl<'a> BlobType<'a> {
  pub const fn as_str(&self) -> &'a str {
    match self {
      Self::OsmHeader => "OSMHeader",
      Self::OsmData => "OSMData",
      Self::Unknown(x) => x,
    }
  }
}

#[derive(Clone, Debug)]
pub enum BlobDecode<'a> {
  OsmHeader(HeaderBlock),
  OsmData(PrimitiveBlock),
  Unknown(&'a str),
}

#[derive(Debug)]
pub struct Blob {
  pub header: proto::BlobHeader,
  pub blob: proto::Blob,
}

fn decode_blob<T: Message + Default>(blob: &proto::Blob) -> error::Result<T> {
  use proto::blob::Data;

  match &blob.data {
    Some(data) => {
      match data {
        Data::ZlibData(v) => {
          let mut decompressed_bytes = Vec::with_capacity(v.len() * 2);
          let mut a = ZlibDecoder::new(&v[..]);
          a.read_to_end(&mut decompressed_bytes).unwrap();
          Ok(T::decode(Bytes::from(decompressed_bytes))?)
        }
        _ => Err(OsmPbfError::UnsupportedCompressedType)
      }
    }
    None => Err(OsmPbfError::DecodeBlobDataError)
  }
}

impl Blob {
  pub fn decode(&self) -> error::Result<BlobDecode> {
    match self.get_type() {
      BlobType::OsmHeader => {
        let header = decode_blob(&self.blob).map(HeaderBlock::new)?;
        Ok(BlobDecode::OsmHeader(header))
      }
      BlobType::OsmData => {
        let block = decode_blob(&self.blob).map(PrimitiveBlock::new)?;
        Ok(BlobDecode::OsmData(block))
      }
      BlobType::Unknown(x) => Ok(BlobDecode::Unknown(x))
    }
  }

  pub fn get_type(&self) -> BlobType {
    match &self.header.r#type {
      x if x == BlobType::OsmHeader.as_str() => BlobType::OsmHeader,
      x if x == BlobType::OsmData.as_str() => BlobType::OsmData,
      x => BlobType::Unknown(x),
    }
  }
}

pub struct BlobReader<R: Read + Send> {
  reader: R,
  finished: bool,
}

impl<R: Read + Send> BlobReader<R> {
  pub fn new(reader: R) -> Self {
    Self { reader, finished: false }
  }

  fn read_blob_header(&mut self) -> Option<error::Result<proto::BlobHeader>> {
    let header_size = match self.reader.read_u32::<byteorder::BigEndian>() {
      Ok(s) => s,
      Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
        self.finished = true;
        return None;
      }
      Err(_) => {
        self.finished = true;
        return Some(Err(OsmPbfError::ReadHeaderSizeError));
      }
    };

    if header_size >= MAX_BLOB_HEADER_SIZE {
      todo!("error")
    }

    let mut buffer: Vec<u8> = Vec::with_capacity(header_size as usize);
    self.reader.by_ref().take(header_size as u64)
      .read_to_end(&mut buffer).expect("TODO: panic message");

    let buffer = Bytes::from(buffer);

    let header = match proto::BlobHeader::decode(buffer) {
      Ok(h) => h,
      Err(e) => {
        return Some(Err(OsmPbfError::DecodeBlobHeaderError));
      }
    };

    Some(Ok(header))
  }
}

impl<R: Read + Send> Iterator for BlobReader<R> {
  type Item = error::Result<Blob>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.finished {
      return None;
    }

    let header = self.read_blob_header();

    let header = match header {
      Some(Ok(header)) => header,
      Some(Err(e)) => return Some(Err(e)),
      None => return None,
    };

    let blob_bytes = self.reader.copy_to_bytes(header.datasize as usize);

    match proto::Blob::decode(blob_bytes) {
      Ok(blob) => Some(Ok(Blob { blob, header })),
      Err(e) => Some(Err(OsmPbfError::DecodeBlobError(e)))
    }
  }
}


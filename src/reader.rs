use std::io::Read;
use crate::blob::{BlobDecode, BlobReader};
use crate::elements::Element;
use crate::error::OsmResult;

pub struct OsmPbfReader<R: Read + Send> {
  pub blob_reader: BlobReader<R>,
}

impl<R: Read + Send> OsmPbfReader<R> {
  pub fn new(reader: R) -> OsmPbfReader<R> {
    OsmPbfReader {
      blob_reader: BlobReader::new(reader),
    }
  }

  pub fn for_each<F>(self, mut f: F) -> OsmResult<()>
  where
    F: FnMut(Element),
  {
    for blob in self.blob_reader {
      match blob?.decode() {
        Ok(BlobDecode::OsmData(block)) => {
          block.for_each(&mut f);
        }
        Ok(_) => {}
        Err(e) => return Err(e),
      }
    }

    Ok(())
  }

  pub fn par_for_each(self) {}
}

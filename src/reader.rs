use std::io::Read;
use prost::Message;
use crate::blob::{BlobDecode, BlobReader, BlobType};
use crate::elements::Element;

pub struct OsmPbfReader<R: Read + Send> {
  blob_reader: BlobReader<R>,
}

impl<R: Read + Send> OsmPbfReader<R> {
  pub fn new(reader: R) -> Self {
    let blob_reader = BlobReader::new(reader);

    Self { blob_reader }
  }

  pub fn for_each<F>(&self, f: F)
  where
    F: FnMut(Element),
  {
    for blob in self.blob_reader {
      match blob {
        Ok(blob) => {
          match blob.get_type() {
            BlobType::OsmData => {
              match blob.decode().unwrap() {
                BlobDecode::OsmData(block) => {
                  for group in block.groups() {
                    for dense in group.dense_node() {
                      f(Element::Node(dense))
                    }

                    for node in group.nodes() {
                      f(Element::Node(node))
                    }

                    for way in group.ways() {
                      f(Element::Way(way))
                    }

                    for relation in group.relations() {
                      f(Element::Relation(relation))
                    }
                  }
                }
                _ => {}
              }
            }
            BlobType::OsmHeader | BlobType::Unknown(_) => {}
          }
        }
        _ => {}
      }
    }
  }
}

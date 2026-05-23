use crate::elements::{Node};
use crate::proto;

pub struct DenseNodeIterator<'a> {
  block: &'a proto::PrimitiveBlock,
  id_iter: std::slice::Iter<'a, i64>,
  lat_iter: std::slice::Iter<'a, i64>,
  lon_iter: std::slice::Iter<'a, i64>,
  last_id: i64,
  last_lat: i64,
  last_lon: i64,
  keys_vals: &'a Vec<i32>,
  keys_vals_index: usize,
}

impl<'a> DenseNodeIterator<'a> {
  pub fn new(block: &'a proto::PrimitiveBlock, dense_node: &'a proto::DenseNodes) -> Self {
    Self {
      block,
      id_iter: dense_node.id.iter(),
      lat_iter: dense_node.lat.iter(),
      lon_iter: dense_node.lon.iter(),
      last_id: 0,
      last_lat: 0,
      last_lon: 0,
      keys_vals: &dense_node.keys_vals,
      keys_vals_index: 0,
    }
  }
}

impl Iterator for DenseNodeIterator<'_> {
  type Item = Node;

  fn next(&mut self) -> Option<Self::Item> {
    let start_index = self.keys_vals_index;
    let mut end_index = start_index;

    for chunk in self.keys_vals[self.keys_vals_index..].chunks(2) {
      if chunk[0] != 0 && chunk.len() == 2 {
        end_index += 2;
        self.keys_vals_index += 2;
      } else {
        self.keys_vals_index += 1;
        break;
      }
    }

    match (self.id_iter.next(), self.lat_iter.next(), self.lon_iter.next()) {
      (Some(id), Some(lat), Some(lon)) => {
        self.last_id += id;
        self.last_lat += lat;
        self.last_lon += lon;


        Some(Node::from_dense(
          self.block,
          self.last_id,
          self.last_lat,
          self.last_lon,
          &self.keys_vals[start_index..end_index],
        ))
      }
      _ => {
        None
      }
    }
  }
}

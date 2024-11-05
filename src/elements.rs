use crate::{error, proto};
use crate::proto::relation::MemberType;

#[derive(Debug, Clone)]
pub enum Element {
  Node(Node),
  Way(Way),
  Relation(Relation),
}

#[derive(Debug, Clone)]
struct Tags(flat_map::FlatMap<String, String>);

impl Tags {
  fn new(block: &proto::PrimitiveBlock, keys: &Vec<u32>, vals: &Vec<u32>) -> Self {
    let mut map = flat_map::FlatMap::with_capacity(keys.len() / 2);
    let mut key_iter = keys.iter();
    let mut vals_iter = vals.iter();

    while let (Some(key), Some(value)) = (key_iter.next(), vals_iter.next()) {
      if let (Ok(key), Ok(value)) = (str_from_stringtable(block, *key as usize), str_from_stringtable(block, *value as usize),) {
        map.insert(key.to_string(), value.to_string());
      }
    }

    Self(map)
  }

  fn from_dense_keys_vals(block: &proto::PrimitiveBlock, keys_vals_indices: &[i32]) -> Self {
    let mut keys_vals_indices_iter = keys_vals_indices.iter();
    let mut keys = Vec::with_capacity(keys_vals_indices.len());
    let mut vals = Vec::with_capacity(keys_vals_indices.len());

    while let (Some(key), Some(value)) = (keys_vals_indices_iter.next(), keys_vals_indices_iter.next()) {
      keys.push(*key as u32);
      vals.push(*value as u32);
    }

    Self::new(block, &keys, &vals)
  }
}

#[derive(Debug, Clone)]
pub struct Node {
  pub id: i64,
  pub nano_lat: i64,
  pub nano_lon: i64,
  pub tags: Tags,
}

impl Node {
  pub(crate) fn new(block: &proto::PrimitiveBlock, id: i64, lat: i64, lon: i64, tags: Tags) -> Self {
    let nano_lat = block.lat_offset() + (lat * block.granularity() as i64);
    let nano_lon = block.lon_offset() + (lon * block.granularity() as i64);

    Self {
      id,
      nano_lat,
      nano_lon,
      tags,
    }
  }

  pub(crate) fn from_node(block: &proto::PrimitiveBlock, node: &proto::Node) -> Self {
    let id = node.id;
    let lat = node.lat;
    let lon = node.lon;

    let tags = Tags::new(block, &node.keys, &node.vals);

    Self::new(block, id, lat, lon, tags)
  }

  pub(crate) fn from_dense(block: &proto::PrimitiveBlock, id: i64, lat: i64, lon: i64, keys_vals_indices: &[i32]) -> Self {
    let tags = Tags::from_dense_keys_vals(block, keys_vals_indices);

    Self::new(block, id, lat, lon, tags)
  }

  pub fn lat(&self) -> f64 {
    1e-9 * self.nano_lat as f64
  }

  pub fn lon(&self) -> f64 {
    1e-9 * self.nano_lon as f64
  }
}

#[derive(Debug, Clone)]
pub struct Way {
  pub id: i64,
  pub nodes: Vec<i64>,
  pub tags: Tags,
}

impl Way {
  pub fn new(block: &proto::PrimitiveBlock, way: &proto::Way) -> Self {
    let mut last_value = 0;
    let nodes = way.refs.iter()
      .map(|x| {
        last_value += x;
        last_value
      })
      .collect::<Vec<i64>>();

    let tags = Tags::new(block, &way.keys, &way.vals);

    Self {
      id: way.id,
      nodes,
      tags: tags,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Relation {
  pub id: i64,
  pub tags: Tags,
  pub(crate) members: Vec<RelMember>,
}

impl Relation {
  pub(crate) fn new(block: &proto::PrimitiveBlock, relation: &proto::Relation) -> Self {
    let tags = Tags::new(block, &relation.keys, &relation.vals);

    let members = Relation::members(block, relation);

    Self {
      id: relation.id,
      tags,
      members,
    }
  }

  fn members(block: &proto::PrimitiveBlock, rel: &proto::Relation) -> Vec<RelMember> {
    RelationMemberIter::new(block, rel).map(|el| el).collect::<Vec<_>>()
  }
}

struct RelationMemberIter<'a> {
  block: &'a proto::PrimitiveBlock,

  member_ids: std::slice::Iter<'a, i64>,
  roles_sid: std::slice::Iter<'a, i32>,
  member_types: std::slice::Iter<'a, i32>,
  last_member_id: i64,
}

impl<'a> RelationMemberIter<'a> {
  fn new(block: &'a proto::PrimitiveBlock, rel: &'a proto::Relation) -> Self {
    Self {
      block,
      member_ids: rel.memids.iter(),
      roles_sid: rel.roles_sid.iter(),
      member_types: rel.types.iter(),
      last_member_id: 0,
    }
  }
}

impl Iterator for RelationMemberIter<'_> {
  type Item = RelMember;

  fn next(&mut self) -> Option<Self::Item> {
    if let (Some(member_id), Some(role), Some(type_)) = (self.member_ids.next(), self.roles_sid.next(), self.member_types.next()) {
      self.last_member_id += member_id;

      let member = match proto::relation::MemberType::try_from(*type_) {
        Ok(MemberType::Node) => ElementId::Node(self.last_member_id),
        Ok(MemberType::Way) => ElementId::Way(self.last_member_id),
        Ok(MemberType::Relation) => ElementId::Relation(self.last_member_id),
        _ => return None,
      };

      let role = str_from_stringtable(self.block, *role as usize).expect("TODO ERROR").to_string();

      return Some(RelMember {
        member,
        role,
      });
    };

    None
  }
}

#[derive(Debug, Clone)]
pub struct RelMember {
  pub member: ElementId,
  pub role: String,
}

#[derive(Debug, Clone)]
pub enum ElementId {
  Node(i64),
  Way(i64),
  Relation(i64),
}

pub(crate) fn str_from_stringtable(block: &proto::PrimitiveBlock, index: usize) -> error::Result<&str> {
  use std::str::from_utf8;

  if let Some(str) = block.stringtable.s.get(index) {
    if let Ok(str) = from_utf8(str) {
      Ok(str)
    } else {
      Err(error::OsmPbfError::TodoError)
    }
  } else {
    Err(error::OsmPbfError::TodoError)
  }
}

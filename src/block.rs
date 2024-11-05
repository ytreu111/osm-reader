use crate::dense::DenseNodeIterator;
use crate::elements::{Node, Relation, Way};
use crate::proto;

#[derive(Debug, Clone)]
pub struct HeaderBlock {
  header: proto::HeaderBlock,
}

impl HeaderBlock {
  pub(crate) fn new(header: proto::HeaderBlock) -> Self {
    Self { header }
  }
}

#[derive(Debug, Clone)]
pub struct PrimitiveBlock {
  pub block: proto::PrimitiveBlock,
}

impl PrimitiveBlock {
  pub(crate) fn new(block: proto::PrimitiveBlock) -> Self {
    Self { block }
  }

  pub fn groups(&self) -> PrimitiveGroupIterator {
    PrimitiveGroupIterator::new(&self.block)
  }
}

pub struct PrimitiveGroupIterator<'a> {
  block: &'a proto::PrimitiveBlock,
  primitivegroup: std::slice::Iter<'a, proto::PrimitiveGroup>,
}

impl<'a> PrimitiveGroupIterator<'a> {
  pub fn new(block: &'a proto::PrimitiveBlock) -> Self {
    Self {
      block,
      primitivegroup: block.primitivegroup.iter(),
    }
  }
}

impl<'a> Iterator for PrimitiveGroupIterator<'a> {
  type Item = PrimitiveGroup<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    let group = self.primitivegroup.next()?;

    Some(PrimitiveGroup::new(&self.block, group))
  }
}

#[derive(Debug, Clone)]
pub struct PrimitiveGroup<'a> {
  pub block: &'a proto::PrimitiveBlock,
  pub group: &'a proto::PrimitiveGroup,
}

impl<'a> PrimitiveGroup<'a> {
  pub(crate) fn new(block: &'a proto::PrimitiveBlock, group: &'a proto::PrimitiveGroup) -> Self {
    Self { block, group }
  }

  pub fn dense_node(&self) -> DenseNodeIterator {
    DenseNodeIterator::new(self.block, self.group.dense.as_ref().unwrap_or_default())
  }

  pub fn nodes(&self) -> ElementNodeIter {
    ElementNodeIter::new(self.block, self.group)
  }

  pub fn ways(&self) -> ElementWayIter {
    ElementWayIter::new(self.block, self.group)
  }

  pub fn relations(&self) -> ElementRelationIter {
    ElementRelationIter::new(self.block, self.group)
  }
}

pub struct ElementOsmNodeIter<'a> {
  block: &'a proto::PrimitiveBlock,
  nodes: std::slice::Iter<'a, proto::Node>,
}

pub(crate) struct ElementNodeIter<'a> {
  block: &'a proto::PrimitiveBlock,
  nodes: std::slice::Iter<'a, proto::Node>,
}

impl<'a> ElementNodeIter<'a> {
  pub(crate) fn new(block: &'a proto::PrimitiveBlock, group: &'a proto::PrimitiveGroup) -> Self {
    Self { block, nodes: group.nodes.iter() }
  }
}

impl Iterator for ElementNodeIter<'_> {
  type Item = Node;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.nodes.next() {
      return Some(Node::from_node(self.block, node));
    }

    None
  }
}

pub(crate) struct ElementWayIter<'a> {
  block: &'a proto::PrimitiveBlock,
  ways: std::slice::Iter<'a, proto::Way>,
}

impl<'a> ElementWayIter<'a> {
  pub(crate) fn new(block: &'a proto::PrimitiveBlock, group: &'a proto::PrimitiveGroup) -> Self {
    Self { block, ways: group.ways.iter() }
  }
}

impl Iterator for ElementWayIter<'_> {
  type Item = Way;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(way) = self.ways.next() {
      return Some(Way::new(self.block, way));
    }

    None
  }
}

pub(crate) struct ElementRelationIter<'a> {
  block: &'a proto::PrimitiveBlock,
  relations: std::slice::Iter<'a, proto::Relation>,
}

impl<'a> ElementRelationIter<'a> {
  pub(crate) fn new(block: &'a proto::PrimitiveBlock, group: &'a proto::PrimitiveGroup) -> Self {
    Self { block, relations: group.relations.iter() }
  }
}

impl<'a> Iterator for ElementRelationIter<'a> {
  type Item = Relation;

  fn next(&mut self) -> Option<Self::Item> {
    Some(Relation::new(self.block, self.relations.next()?))
  }
}

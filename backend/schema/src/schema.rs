use std::collections::HashMap;

use super::Namespace;
use indextree::{Arena, NodeId};

#[derive(Debug)]
pub struct Schema {
    mapping: HashMap<String, NodeId>,
    arena: Arena<Namespace>,
}

impl Schema {
    pub fn new(arena: Arena<Namespace>) -> Self {
        Schema {
            mapping: build_mapping(&arena),
            arena,
        }
    }

    pub fn get_namespace(&self, name: &str) -> Option<&Namespace> {
        match self.mapping.get(name) {
            Some(id) => match self.arena.get(*id) {
                Some(node) => Some(node.get()),
                None => None,
            },
            None => None,
        }
    }
}

fn build_mapping(arena: &Arena<Namespace>) -> HashMap<String, NodeId> {
    let nodes: Vec<(String, NodeId)> = arena
        .iter()
        .filter(|node| !node.is_removed())
        .map(|node| (node.get().name.clone(), arena.get_node_id(node).unwrap()))
        .collect();

    let mut mapping = HashMap::new();

    for (ns, node_id) in nodes {
        assert!(mapping.insert(ns, node_id).is_none());
    }

    mapping
}

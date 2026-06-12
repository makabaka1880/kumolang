// Created by Sean L. on Jun. 12.
// Last Updated by Sean L. on Jun. 12.
//
// kumolang
// src/DAG.rs
//
// Makabaka1880, 2026. All rights reserved.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{
    Serialize,
    ser::{SerializeStruct, SerializeStructVariant},
};
use uuid::Uuid;

fn uuid_to_hex(id: &[u8; 16]) -> String {
    id.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Debug)]
pub enum RecipeNodeType {
    Mixture(),
    Ingredient(Arc<str>),
    Container(Arc<str>),
}

impl Serialize for RecipeNodeType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RecipeNodeType::Mixture() => serializer.serialize_unit_variant("RecipeNodeType", 0, "mixture"),
            RecipeNodeType::Ingredient(s) => {
                let mut v = serializer.serialize_struct_variant("RecipeNodeType", 1, "ingredient", 1)?;
                v.serialize_field("mixture", &**s)?;
                v.end()
            }
            RecipeNodeType::Container(s) => {
                let mut v = serializer.serialize_struct_variant("RecipeNodeType", 2, "container", 1)?;
                v.serialize_field("name", &**s)?;
                v.end()
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RecipeNode {
    #[serde(serialize_with = "serialize_uuid")]
    id: [u8; 16],
    #[serde(serialize_with = "serialize_arc_str")]
    name: Arc<str>,
    #[serde(rename = "node_type")]
    nodeType: RecipeNodeType,
}

fn serialize_uuid<S: serde::Serializer>(id: &[u8; 16], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&uuid_to_hex(id))
}

fn serialize_arc_str<S: serde::Serializer>(val: &Arc<str>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(val)
}

#[derive(Debug)]
pub struct RecipeDAG {
    nodes: Vec<RecipeNode>,
    edges: HashMap<[u8; 16], Vec<[u8; 16]>>,
    /// Maps a user-facing name (mixture or container) to its most recent node id.
    name_index: HashMap<String, [u8; 16]>,
}

impl RecipeDAG {
    pub fn new() -> RecipeDAG {
        RecipeDAG {
            nodes: vec![],
            edges: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    /// Look up the node id for a previously-declared name (mixture or container).
    pub fn getNodeId(&self, name: &str) -> Option<[u8; 16]> {
        self.name_index.get(name).copied()
    }

    /// Create a Mixture node and index it by name.
    pub fn declareMixture(&mut self, name: &str) -> [u8; 16] {
        let id = *Uuid::new_v4().as_bytes();
        self.name_index.insert(name.to_string(), id);
        self.nodes.push(RecipeNode {
            id,
            name: Arc::from(name),
            nodeType: RecipeNodeType::Mixture(),
        });
        id
    }

    /// Create an Ingredient node that references a mixture by name.
    /// Each usage (prep, pour-out, etc.) produces a distinct ingredient instance.
    pub fn declareIngredient(&mut self, mixture_name: &str) -> [u8; 16] {
        let id = *Uuid::new_v4().as_bytes();
        self.nodes.push(RecipeNode {
            id,
            name: Arc::from(mixture_name),
            nodeType: RecipeNodeType::Ingredient(Arc::from(mixture_name)),
        });
        id
    }

    /// Create a Container node and index it by name.
    pub fn declareContainer(&mut self, name: &str) -> [u8; 16] {
        let id = *Uuid::new_v4().as_bytes();
        self.name_index.insert(name.to_string(), id);
        self.nodes.push(RecipeNode {
            id,
            name: Arc::from(name),
            nodeType: RecipeNodeType::Container(Arc::from(name)),
        });
        id
    }

    /// Add a directed edge `from → to` (from depends on to).
    pub fn addEdge(&mut self, from: [u8; 16], to: [u8; 16]) {
        self.edges.entry(from).or_default().push(to);
    }
}

impl Serialize for RecipeDAG {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Convert edges: [u8;16] keys → hex strings
        let edges_serializable: HashMap<String, Vec<String>> = self
            .edges
            .iter()
            .map(|(k, v)| {
                (
                    uuid_to_hex(k),
                    v.iter().map(uuid_to_hex).collect(),
                )
            })
            .collect();

        let mut s = serializer.serialize_struct("RecipeDAG", 2)?;
        s.serialize_field("nodes", &self.nodes)?;
        s.serialize_field("edges", &edges_serializable)?;
        s.end()
    }
}

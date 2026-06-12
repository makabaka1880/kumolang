// Created by Sean L. on Jun. 12.
// Last Updated by Sean L. on Jun. 12.
//
// kumolang
// src/DAG.rs
//
// Makabaka1880, 2026. All rights reserved.

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub enum RecipeNodeType {
    Mixture(Arc<str>),
    Ingredient(Arc<str>, Arc<str>),
    Container(Arc<str>, Arc<str>),
}

#[derive(Debug)]
pub struct RecipeNode {
    id: [u8; 16],
    nodeType: RecipeNodeType,
}

#[derive(Debug)]
pub struct RecipeDAG {
    nodes: Vec<RecipeNode>,
    edges: HashMap<[u8; 16], Vec<[u8; 16]>>,
}

impl RecipeDAG {
    pub fn new() -> RecipeDAG {
        RecipeDAG {
            nodes: vec![],
            edges: HashMap::new(),
        }
    }
    pub fn declareMixture(&mut self, name: &str) {
        self.nodes.push(RecipeNode {
            id: Uuid::new_v4().as_bytes().clone(),
            nodeType: RecipeNodeType::Mixture(Arc::from(name)),
        })
    }
    pub fn declarePrep(&mut self, name: &str) {
        let id = Uuid::new_v4().as_bytes();
        self.nodes.push(RecipeNode { id: id.clone(), nodeType: RecipeNodeType::Container })
    }
}

// Created by Sean L. on Jun. 11.
// Last Updated by Sean L. on Jun. 11.
//
// kumolang
// src/AST.rs
//
// Makabaka1880, 2026. All rights reserved.

#[derive(Debug, Clone)]
pub enum Stmt {
    Skip,
    NewMixture(String),
    Prep(String, String),
    Pour(String, String, String),
    PourOut(String, String, String),
}

#[derive(Debug, Clone)]
pub struct Program(pub Vec<Stmt>);

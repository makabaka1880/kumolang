// Created by Sean L. on Jun. 11.
//
// kumolang
// src/lib.rs
//
// Makabaka1880, 2026. All rights reserved.

#[path = "AST.rs"]
mod ast;
mod engine;

use wasm_bindgen::prelude::*;

/// Verify a KumoLang program from its source string.
/// Returns the same human-readable output as the CLI.
#[wasm_bindgen]
pub fn verify_kumolang(source: &str) -> String {
    match engine::parse_program(source) {
        Ok(program) => {
            let result = engine::verify(&program);
            format!("{}", result)
        }
        Err(e) => format!("Parse error: {}", e),
    }
}

// Created by Sean L. on Jun. 11.
//
// kumolang
// src/lib.rs
//
// Makabaka1880, 2026. All rights reserved.

mod ast;
mod engine;
mod dag;

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

/// Verify a KumoLang program and return the full VerificationResult
/// as pretty-printed JSON, including the provenance graph (nodes + edges).
#[wasm_bindgen]
pub fn verify_kumolang_json(source: &str) -> String {
    match engine::parse_program(source) {
        Ok(program) => {
            let result = engine::verify(&program);
            serde_json::to_string_pretty(&result).unwrap_or_else(|e| {
                format!(r#"{{"passed":false,"errors":["serialization failed: {}"]}}"#, e)
            })
        }
        Err(e) => {
            format!(r#"{{"passed":false,"errors":["parse error: {}"]}}"#, e)
        }
    }
}

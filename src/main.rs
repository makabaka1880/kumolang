// Created by Sean L. on Jun. 11.
// Last Updated by Sean L. on Jun. 11.
//
// kumolang
// src/main.rs
//
// Makabaka1880, 2026. All rights reserved.

#[path = "AST.rs"]
mod ast;
mod engine;

use std::io::Read;

fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read from stdin");

    if input.trim().is_empty() {
        eprintln!("No input provided.");
        std::process::exit(1);
    }

    let program = match engine::parse_program(&input) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let result = engine::verify(&program);
    print!("{}", result);

    if !result.passed {
        std::process::exit(1);
    }
}

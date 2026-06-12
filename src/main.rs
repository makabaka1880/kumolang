// Created by Sean L. on Jun. 11.
// Last Updated by Sean L. on Jun. 12.
//
// kumolang
// src/main.rs
//
// Makabaka1880, 2026. All rights reserved.

mod ast;
mod dag;
mod engine;

use std::fs;
use std::io::{self, Read, Write};
use std::process;

use clap::Parser;

/// KumoLang — a domain-aware recipe verification tool.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Output a JSON-serialized dependency graph and verification result
    #[arg(long, default_value_t = false)]
    structured: bool,

    /// Read program from this file instead of stdin
    #[arg(long, short)]
    input: Option<String>,

    /// Write output to this file instead of stdout
    #[arg(long, short)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let input = read_input(&args).unwrap_or_else(|e| {
        eprintln!("Error reading input: {}", e);
        process::exit(1);
    });

    if input.trim().is_empty() {
        eprintln!("No input provided.");
        process::exit(1);
    }

    let program = match engine::parse_program(&input) {
        Ok(p) => p,
        Err(e) => {
            write_output(&args, &format!("Parse error: {}", e), true);
            process::exit(1);
        }
    };

    let result = engine::verify(&program);

    if args.structured {
        let json = serde_json::to_string_pretty(&result).unwrap_or_else(|e| {
            format!(r#"{{"passed":false,"errors":["serialization failed: {}"]}}"#, e)
        });
        write_output(&args, &json, false);
    } else {
        let text = result.to_string();
        write_output(&args, &text, false);
    }

    if !result.passed {
        process::exit(1);
    }
}

fn read_input(args: &Args) -> io::Result<String> {
    match &args.input {
        Some(path) => fs::read_to_string(path),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn write_output(args: &Args, content: &str, is_error: bool) {
    match &args.output {
        Some(path) => {
            if let Err(e) = fs::write(path, content) {
                eprintln!("Error writing output to {}: {}", path, e);
                process::exit(1);
            }
        }
        None => {
            if is_error {
                io::stderr().write_all(content.as_bytes()).ok();
                io::stderr().write_all(b"\n").ok();
            } else {
                io::stdout().write_all(content.as_bytes()).ok();
            }
        }
    }
}

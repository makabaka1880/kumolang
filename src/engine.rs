// Created by Sean L. on Jun. 11.
// Last Updated by Sean L. on Jun. 11.
//
// kumolang
// src/engine.rs
//
// Makabaka1880, 2026. All rights reserved.

use std::collections::HashMap;

use sexp::{Atom, Sexp};

use crate::ast::*;

// MARK: Store
#[derive(Debug, Clone)]
pub enum Value {
    Mixture(u64),
    Container(Vec<u64>),
}


#[derive(Debug)]
pub struct Store {
    bindings: HashMap<String, Value>,
    next_id: u64,
}

impl Store {
    pub fn new() -> Self {
        Store { bindings: HashMap::new(), next_id: 0 }
    }

    /// Allocate a fresh, unique mixture domain token.
    pub fn fresh_mixture(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }
}


// MARK: Parsing
fn parse_ident(sexp: &Sexp) -> Result<String, String> {
    match sexp {
        Sexp::Atom(Atom::S(s)) => Ok(s.clone()),
        other => Err(format!("expected identifier, got {:?}", other)),
    }
}

fn parse_stmt(sexp: &Sexp) -> Result<Stmt, String> {
    match sexp {
        Sexp::Atom(Atom::S(s)) if s == "skip" => Ok(Stmt::Skip),
        Sexp::List(list) if !list.is_empty() => {
            let op = parse_ident(&list[0])?;
            match op.as_str() {
                "new-mixture" => {
                    if list.len() != 2 {
                        return Err(format!(
                            "new-mixture expects 1 argument, got {}",
                            list.len() - 1
                        ));
                    }
                    Ok(Stmt::NewMixture(parse_ident(&list[1])?))
                }
                "prep" => {
                    if list.len() != 3 {
                        return Err(format!(
                            "prep expects 2 arguments (dest source), got {}",
                            list.len() - 1
                        ));
                    }
                    Ok(Stmt::Prep(parse_ident(&list[2])?, parse_ident(&list[1])?))
                }
                "pour" => {
                    if list.len() != 4 {
                        return Err(format!(
                            "pour expects 3 arguments (dest src1 src2), got {}",
                            list.len() - 1
                        ));
                    }
                    Ok(Stmt::Pour(
                        parse_ident(&list[2])?,
                        parse_ident(&list[3])?,
                        parse_ident(&list[1])?,
                    ))
                }
                "skip" => {
                    if list.len() != 1 {
                        return Err(format!("skip expects 0 arguments, got {}", list.len() - 1));
                    }
                    Ok(Stmt::Skip)
                }
                "pour-out" => {
                    if list.len() != 4 {
                        return Err(format!(
                            "pour-out expects 3 arguments (dest container target), got {}",
                            list.len() - 1
                        ));
                    }
                    Ok(Stmt::PourOut(
                        parse_ident(&list[2])?,
                        parse_ident(&list[3])?,
                        parse_ident(&list[1])?,
                    ))
                }
                other => Err(format!("unknown statement '{}'", other)),
            }
        }
        other => Err(format!("expected a statement, got {:?}", other)),
    }
}

/// Parse a complete KumoLang program from an S-expression string.
pub fn parse_program(input: &str) -> Result<Program, String> {
    let parsed = sexp::parse(input).map_err(|e| format!("parse error: {}", e))?;

    let stmts = match &parsed {
        Sexp::List(list) if !list.is_empty() => {
            if let Sexp::Atom(Atom::S(s)) = &list[0] {
                if s == "begin" || s == "program" {
                    // Skip the wrapper symbol
                    list[1..].iter().map(parse_stmt).collect::<Result<Vec<_>, _>>()?
                } else {
                    list.iter().map(parse_stmt).collect::<Result<Vec<_>, _>>()?
                }
            } else {
                list.iter().map(parse_stmt).collect::<Result<Vec<_>, _>>()?
            }
        }
        // A single top-level statement (e.g. just `skip`)
        _ => vec![parse_stmt(&parsed)?],
    };

    Ok(Program(stmts))
}

// MARK: Result
#[derive(Debug)]
pub struct VerificationResult {
    pub passed: bool,
    pub errors: Vec<String>,
    #[allow(dead_code)]
    pub store: Store,
}

impl std::fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.passed {
            writeln!(f, "✓ Verification passed.")?;
        } else {
            for err in &self.errors {
                writeln!(f, "✗ {}", err)?;
            }
            writeln!(f, "Verification FAILED — {} error(s).", self.errors.len())?;
        }
        Ok(())
    }
}

// MARK: Interpreter
/// Run the full verification pipeline over a parsed program.
pub fn verify(program: &Program) -> VerificationResult {
    let mut store = Store::new();
    let mut errors: Vec<String> = Vec::new();

    for stmt in &program.0 {
        if let Err(msg) = eval_stmt(stmt, &mut store) {
            errors.push(msg);
        }
    }

    VerificationResult { passed: errors.is_empty(), errors, store }
}

fn eval_stmt(stmt: &Stmt, store: &mut Store) -> Result<(), String> {
    match stmt {
        Stmt::Skip => Ok(()),

        Stmt::NewMixture(name) => {
            let mid = store.fresh_mixture();
            store.set(name.clone(), Value::Mixture(mid));
            Ok(())
        }

        Stmt::Prep(source, dest) => match store.get(source) {
            Some(Value::Mixture(m)) => {
                store.set(dest.clone(), Value::Container(vec![*m]));
                Ok(())
            }
            Some(Value::Container(_)) => Err(format!(
                "prep: '{}' is a container, expected a mixture",
                source
            )),
            None => Err(format!("prep: '{}' is not bound", source)),
        },

        Stmt::Pour(src1, src2, dest) => match (store.get(src1), store.get(src2)) {
            (Some(Value::Container(c1)), Some(Value::Container(c2))) => {
                let domain_violation = match (c1.first(), c2.first()) {
                    (Some(a), Some(b)) if a != b => {
                        Some(format!(
                            "pour: containers '{}' (domain {}) and '{}' (domain {}) differ",
                            src1, a, src2, b
                        ))
                    }
                    _ => None,
                };
                if let Some(msg) = domain_violation {
                    return Err(msg);
                }
                let combined: Vec<u64> = c1.iter().chain(c2.iter()).copied().collect();
                store.set(dest.clone(), Value::Container(combined));
                Ok(())
            }
            (Some(Value::Mixture(_)), _) => Err(format!(
                "pour: '{}' is a mixture, expected a container",
                src1
            )),
            (_, Some(Value::Mixture(_))) => Err(format!(
                "pour: '{}' is a mixture, expected a container",
                src2
            )),
            (None, _) => Err(format!("pour: '{}' is not bound", src1)),
            (_, None) => Err(format!("pour: '{}' is not bound", src2)),
        },

        Stmt::PourOut(container, target, dest) => {
            // Classify both arguments first to avoid borrow conflicts.
            let container_kind = store.get(container).map(|v| match v {
                Value::Mixture(_) => "mixture",
                Value::Container(_) => "container",
            });
            let target_kind = store.get(target).map(|v| match v {
                Value::Mixture(_) => "mixture",
                Value::Container(_) => "container",
            });
            let target_mix = match store.get(target) {
                Some(Value::Mixture(m)) => Some(*m),
                _ => None,
            };

            match (container_kind, target_kind, target_mix) {
                (Some("container"), _, Some(m_target)) => {
                    store.set(container.clone(), Value::Container(Vec::new()));
                    store.set(dest.clone(), Value::Container(vec![m_target]));
                    Ok(())
                }
                (Some("mixture"), _, _) => Err(format!(
                    "pour-out: '{}' is a mixture, expected a container",
                    container
                )),
                (None, _, _) => Err(format!("pour-out: '{}' is not bound", container)),
                (Some("container"), None, _) => {
                    Err(format!("pour-out: '{}' is not bound", target))
                }
                (Some("container"), Some("container"), _) => Err(format!(
                    "pour-out: '{}' is a container, expected a mixture",
                    target
                )),
                _ => unreachable!(),
            }
        }
    }
}

# KumoLang

> A formal verification framework for culinary procedure synthesis.

KumoLang is a domain-specific language for describing and verifying culinary procedures through operational semantics and provenance analysis.

Originally developed as part of a joke research paper, KumoLang is nevertheless a fully functional language with a reference implementation written in Rust.

The system models culinary procedures as programs, recipes as execution traces, and synthesized food artifacts as verified outputs. Execution traces may be transformed into dependency graphs for provenance analysis, allergen tracing, and culinary genealogy studies.

## Disclaimer

This project originated as an end-of-term joke paper.

The language itself, however, is real.

The parser, interpreter, operational semantics, and provenance model are all legitimate implementations. The presentation is intentionally absurd.

## Features

* Lisp-like syntax
* Small-step operational semantics
* Explicit resource tracking
* Provenance extraction
* Dependency graph generation
* Allergen tracing
* Culinary genealogy analysis through graph matching (TODO)

## Installation

```bash
cargo build --release
```

## Usage

The reference interpreter reads a KumoLang program from standard input and emits an execution report to standard output.

```bash
cargo run < recipe.kumo
```

or

```bash
cat recipe.kumo | cargo run
```

## Example

```lisp
(begin
    (new-mixture BaseMix)
    (new-mixture BakeMix)

    (prep c_cheese BaseMix)
    (prep c_sugar BaseMix)

    (pour c_bowl c_cheese c_sugar)

    (pour-out c_cake c_bowl BakeMix)

    (skip)
)
```

Example output:

```text
✓ Parsed 7 statements
✓ Executed program
✓ Constructed provenance graph
✓ Verified culinary artifact

Result: Cheesecake synthesis successful.
```

## Language Specification

### Statements

```text
<stmt> ::= <skip_stmt>
         | ( new-mixture <id> )
         | ( prep        <id> <id> )
         | ( pour        <id> <id> <id> )
         | ( pour-out    <id> <id> <id> )
```

### Skip

```text
<skip_stmt> ::= skip
              | ( skip )
```

### Identifiers

```text
<id>
```

Any identifier accepted by `parse_ident`.

## Operational Model

KumoLang manipulates two primary entities:

### Mixtures

A mixture represents a culinary domain.

```lisp
(new-mixture BaseMix)
```

creates a new mixture domain.

### Containers

Containers represent prepared ingredients and intermediate culinary artifacts.

```lisp
(prep c_cheese BaseMix)
```

prepares an ingredient from a mixture.

```lisp
(pour c_bowl c_cheese c_sugar)
```

combines containers.

```lisp
(pour-out c_cake c_bowl BakeMix)
```

materializes an artifact within a target mixture.

## Provenance Analysis

Every execution trace can be transformed into a dependency graph.

This enables:

* ingredient lineage recovery
* allergen source tracing
* contamination analysis
* recipe comparison
* culinary genealogy reconstruction

For example, Matcha Basque Cheesecake and Chocolate Basque Cheesecake may be identified as members of the same culinary family through subgraph isomorphism.

## Theoretical Background

The formal semantics are described in the accompanying paper:

> KumoLang: A Formal Verification Framework for Culinary Procedure Synthesis

The paper contains:

* abstract syntax
* semantic domains
* small-step operational semantics
* trace evaluation
* provenance extraction
* verification procedures

## Limitations

Current verification guarantees do not extend to:

* flavor quality
* texture quality
* oven malfunction
* user error
* existential disappointment
* programmer-manager conflicts

## Acknowledgements

The authors thank Kumo for an excellent APCSA course.

The authors thank KumoKumo for substantial contributions to the experimental benchmark suite.

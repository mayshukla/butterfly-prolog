# Butterfly-Prolog

Butterfly-Prolog is a variant of prolog aimed at being a good language for procedural generation.
I am trying to make a language that is both beautiful and chaotic.

## Design Goals

- Useful for various types of procedural generation (music, graphics, weaving, etc.)
- Embeddable in other languages, apps (e.g. webassembly, call from c, call c)
  - This makes it easier to use for procedural generation since it can directly control midi, graphics, etc.
- Intuitive, ergonomic (quick to type) syntax
- Written in rust (can run as standalone binary or wasm)

## Non-Goals

- General-purpose language
  - There are many better general-purpose languages out there
- Be the fastest prolog implementation
  - Many smart people have spent many years making fast implementations of prolog
- ISO prolog compliance
  - I want to create a simpler syntax

## Syntax Examples

TODO

## Implementation Plan

### Phase 1: Implement prolog VM based on Tarau paper

See Paul Tarau's iProlog: <https://github.com/ptarau/iProlog>

### Phase 2: Add (optional?) random search

Instead of prolog's top-to-bottom, depth-first search, unfold clauses randomly.
This will let you code procedural generation with randomness built in!

This should probably be optional and able to be switched on and off globally or on a per-predicate basis.

### Phase 3: Add arithmetic, functions, native predicates
### Phase 4: Embed in a wasm app, make cffi
### Phase 5: Add type checking
### Phase 6: JIT, performance optimizations

## Project Status

I just started this and I am very much in phase 1!
Nothing is implemented yet!

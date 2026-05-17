# Event models

This directory contains language-specific event model bindings.

Currently:

- Rust: the canonical models live in
  [`shared/src/events.rs`](../src/events.rs).
- TypeScript: generated bindings live in
  [`frontend/src/lib/api/events.ts`](../../frontend/src/lib/api/events.ts).
- JSON Schema: machine-readable versions in
  [`shared/schemas/`](../schemas/).

If you change an event shape, update **all three** in the same commit.

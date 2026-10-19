# Split `sisho-desk-core` into `sisho-desk-core` and `sisho-desk-tui`

Status: completed. Verified 2026-10-20.

## Objective

Split the monolith `crates/sisho-desk-core` into two clean layers:
1. `sisho-desk-core`: Pure data models, state machine (`App`), navigation, and migration logic. Free of terminal/TUI dependencies (`ratatui`, `crossterm`, `tree-sitter`).
2. `sisho-desk-tui`: Terminal driver (`Screen`), event loop, rendering widgets (`ui/*`), syntax highlighting, and diff rendering.

Also remove unused dependencies (`tomet-ast`, `tomet-parser`, `tomet-semantics`, `tomet-tree`) from `sisho-desk-core`.

Note: Worked directly in the default workspace per explicit user permission.

## Steps

- [x] Task defined and approach agreed
- [x] Configure `Cargo.toml` files:
  - Add `crates/sisho-desk-tui` to root workspace members and dependencies
  - Clean up `crates/sisho-desk-core/Cargo.toml` (remove unused `tomet-*` crates, remove `ratatui`, `crossterm`, `tree-sitter`)
  - Create `crates/sisho-desk-tui/Cargo.toml`
  - Update `app/Cargo.toml` to depend on `sisho-desk-tui`
- [x] Restructure code:
  - Create directory `crates/sisho-desk-tui/src`
  - Move `ui/` and `render_tests.rs` to `crates/sisho-desk-tui/src/`
  - Implement terminal driver / event loop in `sisho-desk-tui/src/lib.rs`
  - Clean up `sisho-desk-core/src/lib.rs` to expose pure domain & state models (`app`, `engine`)
  - Update `app/src/main.rs` to invoke `sisho_desk_tui::run_tui`
- [x] Update `.writ.tmt`:
  - Update `@rule(crate-layering)` layers to:
    ```
    layers: {
      2: list( "app" )
      1: list( "crates/sisho-desk-tui" )
      0: list( "crates/sisho-desk-core" )
    }
    ```
- [x] Verification:
  - `cargo test --workspace`: 16 passed, 0 failed
  - `cargo clippy --workspace --all-targets -- -D warnings`: passed cleanly
  - `cargo fmt --check`: passed cleanly
  - `just docs-check`: passed cleanly (`twrit check`: ok)

# Tidy the tabs of `sisho-desk`

Status: not started. Recorded 2026-10-20, at the end of the move of `sisho tui` out of `sisho` (that move kept the behaviour as it was on purpose, all four tabs). This task is the deliberate change that the move did not make. The decisions that need the author are marked **Ask**.

## Where things stand

`sisho-desk` has four tabs: Explorer, Migration, BatchMeta, StructuralGrep (`crates/sisho-desk-core/src/app/*`, `ui/*`). It came from `tomet tui` and has had no redesign since.

- **StructuralGrep** searches and replaces by syntax with `tomet-transform` (`app/structural.rs`). The sisho design memo (2026-10-18) says `tandr` is meant to take over that ground, and `tandr` has no command line yet. So this tab is expected to go, but not before there is something to replace it.
- **Migration** (Markdown to `.tmt`) and **BatchMeta** (one `@meta` field over many documents) are what someone who has just started with Tomet wants first. They need Tomet knowledge, so they are the part the community is least likely to replace. They are this extension's reason to exist.
- **Explorer** is the part most likely to be replaced by better community extensions (`sisho-explorer`, ...). The earlier recommendation was a minimal one, kept as the way to pick files for the other tabs. Not decided.
- Seen on 2026-10-20 on an 80x24 virtual terminal: the header showed only three tab titles; the fourth was cut off. Check at which widths a tab title is lost, and fix it whatever the tabs end up being.

## Ask (the author decides; do not decide here)

1. Does StructuralGrep go now, or stay until `tandr` has a command line? If it goes now, say what a person who used it should do (nothing replaces it yet).
2. When `tandr` has a command line, should the desk call it as a subprocess, instead of linking `tomet-transform`? That would follow the rule already in `AGENTS.md` (the desk knows the contract, not another program's crates) and take `tomet-transform` out. Do not start it before `tandr` exists.
3. How much Explorer: keep as is, make it minimal (a file picker that hands the choice to the other tabs), or remove it. What the other tabs need from it decides this.
4. Tab names and order, once the set is known. The names today are inherited ("Workspace Explorer", "Migration (MD -> TM)", "Batch Meta Editor", "StructuralGrep").

## Facts to check before changing anything

- `tree-sitter` and `tree-sitter-tomet` are also used by `ui/highlight.rs` (the preview in Explorer), not only by the structural tab; removing the tab does not remove them. `tomet-transform` appears in `app/structural.rs`; check the rest with `cargo tree`.
- The tests that name a tab (`render_tests.rs`: 7 render tests; `app/mod.rs` unit tests) must change with the tabs.
- The spec `tmtroot/spec/cli/desk.tmt` describes four tabs, the keys and what each writes. It is the contract with the reader; change it in the same step as the code.

## Steps

- [ ] Get the answers to **Ask** 1 to 4 from the author. Write them as decisions at the top of this file.
- [ ] Fix the header so that every tab title is visible at 80 columns, or say at which width it is not and why. Test with `TestBackend` at 80 and at a narrower width.
- [ ] Apply the decisions: remove or reshape tabs, keys, status messages, and the tab strip; keep the confirmation before every write (Migration, BatchMeta, saving in Explorer) exactly as it is.
- [ ] Remove dependencies that nothing uses any more (check each with `cargo tree`); update `Cargo.lock`.
- [ ] Update `tmtroot/spec/cli/desk.tmt`, `tmtroot/readme.tmt` and the help text (`app/src/main.rs`); `just docs`.
- [ ] A way to look at it that can be run again: a tmux script in `scripts/` that starts `sisho-desk` on a fixed vault, presses a series of keys and compares the screens against stored expectations (the parity script of the move is gone: it compared against `sisho tui`, which no longer exists). Show it fails when a screen changes.
- [ ] `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `just docs-check`, `twrit check .`.

## Also open in this repository (not part of the tabs)

- The rule "no `sisho-*` crate is a dependency" is proposed for `.writ.tmt` (a `door` guard with `names` of the `sisho_*` crates, `within: crates/*`, `allowed: list()`; tried on a scratch copy on 2026-10-20: clean copy passes, a `use sisho_run::Step;` fails). Only the author edits `.writ.tmt`. Until it is applied, the design memo of `sisho` must not say it is in place.
- `sisho` has a task for a TUI of its own (`sisho/.agents/tasks/self-tui.md`) and one for listing extensions (`extension-list.md`). Neither changes this repository, except that a way to hand a file from `sisho` to `sisho-desk` is an open question there. If one is wanted, the contract is written down in `sisho` first and implemented here second.

## Not in this task

A plugin protocol for panes inside the desk; making the desk depend on any `sisho` crate; changes to how `sisho desk` finds `sisho-desk` (that is `sisho`'s contract, `tmtroot/spec/cli/extension.tmt` there).

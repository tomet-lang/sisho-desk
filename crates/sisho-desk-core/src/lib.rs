//! Sisho Desk core state management and domain logic.
//!
//! This crate contains the application state machine (`app`) and migration
//! domain logic (`engine`), free of terminal and TUI dependencies.

pub mod app;
pub mod engine;

pub use app::App;

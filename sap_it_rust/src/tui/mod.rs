//! Terminal User Interface module for SAP-IT.
//!
//! Provides an interactive TUI for managing server connections.

pub mod app;
pub mod event;
pub mod ui;

pub use app::App;
pub use event::{Event, EventHandler};

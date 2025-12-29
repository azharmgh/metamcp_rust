//! Streaming module for real-time client communication

pub mod manager;

pub use manager::{EventFilters, SharedStreamManager, StreamEvent, StreamManager};

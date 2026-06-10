//! NMP-owned project feedback.
//!
//! This crate owns the reusable Nostr feedback behavior that app shells should
//! not duplicate: project-scoped kind:1/kind:513 interest construction, explicit
//! feedback-relay publish dispatch through NMP, event observation, bounded event
//! caching, and resolved thread projection.

mod command;
mod config;
mod observer;
mod projection;
mod runtime;

pub use command::{fetch_feedback, publish_feedback, FeedbackCommandOutcome};
pub use config::{FeedbackConfig, DEFAULT_FEEDBACK_RELAY};
pub use observer::{FeedbackEventCache, FeedbackObserver};
pub use projection::{reduce_feedback_threads, FeedbackReplyDto, FeedbackThreadDto};
pub use runtime::{FeedbackRuntime, SnapshotBump};

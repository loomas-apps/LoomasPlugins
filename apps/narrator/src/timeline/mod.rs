//! The events found by other plugins, on one timeline, and which face is whose voice.

mod event_contract;
mod link_faces_policy;

pub use event_contract::{Event, Kind};
pub use link_faces_policy::link_faces;

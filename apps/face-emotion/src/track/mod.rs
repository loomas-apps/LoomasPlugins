//! Following each face from picture to picture, and joining its expressions into runs.

mod expression_runs_algorithm;
mod track_faces_algorithm;

pub use expression_runs_algorithm::{expression_runs, Reading};
pub use track_faces_algorithm::{track_faces, Sighting};

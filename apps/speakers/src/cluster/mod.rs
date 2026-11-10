//! From the local speakers of each window to the voices of the whole sound: the voice prints are
//! grouped, then the frames of each group become turns.

mod build_turns_algorithm;
mod cluster_voices_algorithm;

pub use build_turns_algorithm::{build_turns, frame_middle, Turn};
pub use cluster_voices_algorithm::{cluster_voices, Piece};

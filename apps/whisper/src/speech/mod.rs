//! What whisper.cpp heard, as the sentences (and words) a flow reads.

mod segments_to_sentences_mapper;

pub use segments_to_sentences_mapper::{sentences_of, HeardSegment, HeardToken, Placing};

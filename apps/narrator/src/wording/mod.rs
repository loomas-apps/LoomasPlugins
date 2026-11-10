//! Writing the timeline as sentences: names for the speakers, the templates, and the phrases.

mod describe_line_algorithm;
mod name_people_mapper;
mod wording_config;

pub use describe_line_algorithm::{describe_line, describe_quiet, Said};
pub use name_people_mapper::name_people;
pub use wording_config::{face_phrase, movement_clause, voice_phrase};

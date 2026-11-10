//! The voices kept between runs: the pattern file, reading the ones given, matching them to the
//! voices heard, and learning from new speech.

mod learn_pattern_algorithm;
mod match_voices_policy;
mod read_voice_patterns_use_case;
mod voice_pattern_contract;

pub use learn_pattern_algorithm::learn_pattern;
pub use match_voices_policy::match_voices;
pub use read_voice_patterns_use_case::read_voice_patterns;
pub use voice_pattern_contract::VoicePattern;

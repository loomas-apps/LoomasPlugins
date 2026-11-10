//! The voice print: 256 numbers that are close for the same voice, made by WeSpeaker from Kaldi
//! filter-bank features.

mod filter_bank_algorithm;
mod make_voice_print_use_case;

pub use make_voice_print_use_case::{make_voice_print, unit, PRINT_SIZE};

//! The actions Loomas calls: `transcribe`.

mod read_transcribe_options_mapper;
mod run_method_use_case;
mod transcribe_options_contract;
mod transcribe_use_case;

pub use run_method_use_case::run_method;

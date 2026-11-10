//! Who speaks when inside each window of 10 seconds, as the speech model hears it: up to three
//! local speakers a window, not yet linked to the speakers of other windows.

mod powerset_mapper;
mod segment_windows_use_case;

#[cfg(test)]
pub use powerset_mapper::FrameLabel;
pub use segment_windows_use_case::{segment_windows, Window, FRAME_SPAN, FRAME_STEP, RATE, STEP, WINDOW};

//! Where the speech model runs: the processor, or a graphics card through Vulkan.

mod device_choice_policy;

pub use device_choice_policy::{plan_device, DeviceChoice};

//! The whisper plugin: writes down what is said in a video or a sound, with the time of each
//! sentence, using whisper.cpp. It runs on the processor, or on a graphics card through Vulkan (AMD,
//! NVIDIA, Intel) when built with the `vulkan` feature. See README.md; the protocol is spoken by the
//! SDK in libs/plugin-sdk.

mod device;
mod methods;
mod models;
mod speech;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}

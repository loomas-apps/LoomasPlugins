//! The voice tone plugin: how each stretch of speech in a video is said, how loud against the
//! rest of the video and with which emotion. See README.md; the protocol is spoken by the SDK in
//! libs/plugin-sdk.

mod emotion;
mod loudness;
mod methods;
mod models;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}

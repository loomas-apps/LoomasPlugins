//! The speakers plugin: finds who speaks when in a video, gives each voice a pattern file and a
//! sample to listen to, and names the voices it knows from earlier runs. See README.md; the
//! protocol is spoken by the SDK in libs/plugin-sdk.

mod cluster;
mod methods;
mod models;
mod segment;
mod voiceprint;
mod voices;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}

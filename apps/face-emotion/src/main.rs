//! The face emotion plugin: finds the faces in a video, follows each one, and says which
//! expression it shows over time. See README.md; the protocol is spoken by the SDK in
//! libs/plugin-sdk.

mod detect;
mod expression;
mod methods;
mod models;
mod track;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}

//! The narrator plugin: joins what other plugins found in a video (who said what, how, with which
//! face, what moved, what was in view) into a timeline written as plain sentences. No model of any
//! kind. See README.md; the protocol is spoken by the SDK in libs/plugin-sdk.

mod methods;
mod timeline;
mod wording;

use loomasplugins_plugin_sdk::serve_stdio;

fn main() {
    serve_stdio(methods::run_method);
}

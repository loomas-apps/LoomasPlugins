//! The plugin as Loomas starts it: the handshake, a good call, a wrong call (which must come back
//! as an error and not a result), an action it does not have, and the end.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn talk(messages: &[Value]) -> Vec<Value> {
    let mut plugin = Command::new(env!("CARGO_BIN_EXE_loomasplugins-narrator"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = plugin.stdin.take().unwrap();
    let mut output = BufReader::new(plugin.stdout.take().unwrap());
    let mut answers = Vec::new();
    for message in messages {
        writeln!(input, "{message}").unwrap();
        if message["type"] != "shutdown" {
            let mut line = String::new();
            output.read_line(&mut line).unwrap();
            answers.push(serde_json::from_str(&line).unwrap());
        }
    }
    drop(input);
    plugin.wait().unwrap();
    answers
}

#[test]
fn it_answers_hello_narrates_refuses_a_wrong_call_and_stops_on_shutdown() {
    let answers = talk(&[
        json!({ "type": "hello", "protocol": 2, "capabilities": {} }),
        json!({ "type": "call", "id": 1, "method": "narrate",
                "params": { "turns": [{ "startMs": 0, "endMs": 900, "label": "voice-1" }] } }),
        json!({ "type": "call", "id": 2, "method": "narrate", "params": { "sentences": [{ "startMs": 0 }] } }),
        json!({ "type": "call", "id": 3, "method": "find", "params": {} }),
        json!({ "type": "shutdown" }),
    ]);
    assert_eq!(answers[0]["type"], "ready");
    assert_eq!(answers[1]["type"], "result", "{}", answers[1]);
    assert_eq!(answers[1]["result"]["text"], "Person A spoke.");
    for (answer, code) in answers[2..].iter().zip([400, 404]) {
        assert_eq!(answer["type"], "error", "{answer}");
        assert_eq!(answer["code"], code, "{answer}");
    }
}

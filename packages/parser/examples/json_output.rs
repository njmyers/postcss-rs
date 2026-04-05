// Test to see what JSON format our Rust parser outputs

use postcss_rs_parser::{sinks::TreeSink, ParseSink, Parser};

fn main() {
    let css = ".test { color: red; }";

    let mut sink = TreeSink::new();
    let mut parser = Parser::new(css).unwrap();
    parser.parse(&mut sink);
    let node = sink.finish();

    let json = serde_json::to_string_pretty(&node).unwrap();

    println!("postcss-rs JSON format:");
    println!("{}", json);
}

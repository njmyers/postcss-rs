// Test to see what JSON format our Rust parser outputs

use postcss_rs_parser::Parser;

fn main() {
    let css = ".test { color: red; }";

    let mut parser = Parser::new(css).unwrap();
    let node = parser.parse();

    let json = serde_json::to_string_pretty(&node).unwrap();

    println!("postcss-rs JSON format:");
    println!("{}", json);
}

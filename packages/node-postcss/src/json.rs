use postcss_rs_parser::Node;

/// Serialize `node` to a JSON byte buffer using sonic-rs (SIMD-accelerated).
/// The buffer is pre-sized at 5× the CSS source length to avoid reallocations;
/// CSS AST JSON is typically 3–8× the CSS source size.
pub fn serialize(node: &Node, css: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(css.len() * 5);
    sonic_rs::to_writer(&mut buf, node).expect("sonic_rs serialization failed");
    buf
}

/// Criterion benchmarks for the Rust-side of postcss-rs.
///
/// Suites are grouped by serialization format, mirroring the TypeScript bench's
/// sync / async / raw grouping:
///
///   parse   — pure parse (no serialization), baseline
///   json    — parse + JSON serialize (sonic-rs)
///   offsets — parse + flat offset-array serialize
///
/// Run with:
///   cargo bench --target x86_64-unknown-linux-gnu -p postcss-rs-node-postcss
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use postcss_rs_parser::{offset_sink::OffsetSink, Parser};

// Pull in serialisers without going through the NAPI lib.
#[path = "../src/json.rs"]
#[allow(dead_code)]
mod json;

static SMALL_CSS: &str =
    unsafe { std::str::from_utf8_unchecked(include_bytes!("fixtures/small.css")) };

static MEDIUM_CSS: &str =
    unsafe { std::str::from_utf8_unchecked(include_bytes!("fixtures/medium.css")) };

static LARGE_CSS: &str =
    unsafe { std::str::from_utf8_unchecked(include_bytes!("fixtures/large.css")) };

/// All three size variants in order.
fn fixtures() -> [(&'static str, &'static str); 3] {
    [
        ("small  (~200 B)", SMALL_CSS),
        ("medium (~2 KB)", MEDIUM_CSS),
        ("large  (~30 KB)", LARGE_CSS),
    ]
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    for (label, css) in fixtures() {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                black_box(p.parse())
            })
        });
    }
    group.finish();
}

fn bench_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("json");
    for (label, css) in fixtures() {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                black_box(json::serialize(&node, css))
            })
        });
    }
    group.finish();
}

fn bench_offsets(c: &mut Criterion) {
    let mut group = c.benchmark_group("offsets");
    for (label, css) in fixtures() {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut sink = OffsetSink::new(black_box(css));
                let mut p = Parser::new(black_box(css)).unwrap();
                p.parse_into(&mut sink);
                black_box(sink.finish())
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse, bench_json, bench_offsets);
criterion_main!(benches);

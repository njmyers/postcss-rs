/// Criterion benchmarks for the Rust-side of postcss-rs.
///
/// These measure:
///   - pure parse (postcss-rs-parser, no serialization)
///   - parse + JSON serialize   (sonic-rs)
///   - parse + FlatBuffers      (flatbuffers)
///   - parse + MessagePack      (rmp-serde)
///
/// Run with:
///   cargo bench --target x86_64-unknown-linux-gnu -p postcss-rs-node-postcss
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use postcss_rs_parser::Parser;

// Pull in the FlatBuffers serialiser without going through the NAPI lib.
// The #[path] attribute is resolved relative to *this* file's directory
// (benches/), so "../src/fb_serializer.rs" resolves to src/fb_serializer.rs.
// The nested `#[path = "css_ast_generated.rs"]` inside that file is then
// resolved relative to *its* own directory (src/), which is correct.
#[path = "../src/fb_serializer.rs"]
mod fb_serializer;

// ── CSS Fixtures (same sizes as the TypeScript bench) ────────────────────────

static SMALL_CSS: &str = r#".button {
  display: inline-flex;
  align-items: center;
  padding: 8px 16px;
  background-color: #0070f3;
  color: #fff;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}"#;

static MEDIUM_CSS: &str = r#"
/* Reset */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

:root {
  --color-primary: #0070f3;
  --color-secondary: #7928ca;
  --color-background: #fff;
  --color-text: #111;
  --font-size-base: 16px;
  --spacing-unit: 8px;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, sans-serif;
  font-size: var(--font-size-base);
  color: var(--color-text);
  background: var(--color-background);
  line-height: 1.5;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 calc(var(--spacing-unit) * 2);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: calc(var(--spacing-unit) * 1) calc(var(--spacing-unit) * 2);
  border: none;
  border-radius: 4px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s ease, transform 0.1s ease;
}

.btn-primary {
  background-color: var(--color-primary);
  color: #fff;
}

.btn-primary:hover {
  background-color: #0060df;
}

.btn-primary:active {
  transform: scale(0.97);
}

.card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: calc(var(--spacing-unit) * 3);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.card__title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: calc(var(--spacing-unit) * 1);
}

.card__body {
  color: #6b7280;
  line-height: 1.6;
}

@media (max-width: 768px) {
  .container {
    padding: 0 var(--spacing-unit);
  }

  .card {
    border-radius: 4px;
  }
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-8px); }
  to   { opacity: 1; transform: translateY(0); }
}

@supports (display: grid) {
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: calc(var(--spacing-unit) * 3);
  }
}
"#;

/// Build the "large" CSS fixture at startup.  Mirrors the TS bench's
/// `Array.from({ length: 50 }, (_, i) => ...)` template.
fn make_large_css() -> String {
    (0..50)
        .map(|i| {
            format!(
                r#"
.component-{i} {{
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: {p0}px {p1}px;
  margin: {m}px;
  background-color: hsl({h}, 60%, 95%);
  border: 1px solid hsl({h}, 40%, 80%);
  border-radius: {br}px;
  font-size: {fs}px;
  color: hsl({h}, 40%, 20%);
  transition: all 0.2s ease;
}}

.component-{i}:hover {{
  background-color: hsl({h}, 60%, 88%);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}}

.component-{i}__title {{
  font-size: {ts}px;
  font-weight: 600;
  margin-bottom: 8px;
}}

@media (max-width: {mq}px) {{
  .component-{i} {{
    padding: {p0}px;
  }}
}}
"#,
                i = i,
                p0 = 8 + i,
                p1 = 16 + i,
                m = 4 + i,
                h = i * 7,
                br = 4 + (i % 8),
                fs = 12 + (i % 6),
                ts = 14 + (i % 4),
                mq = 320 + i * 10,
            )
        })
        .collect()
}

// ── Bench helpers ─────────────────────────────────────────────────────────────

fn bench_parse_only(c: &mut Criterion) {
    let large = make_large_css();
    let cases: &[(&str, &str)] = &[
        ("small  (~200 B)", SMALL_CSS),
        ("medium (~2 KB)",  MEDIUM_CSS),
    ];

    let mut group = c.benchmark_group("parse_only");
    for (label, css) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                black_box(p.parse())
            })
        });
    }
    group.bench_with_input(
        BenchmarkId::from_parameter("large  (~20 KB)"),
        &large,
        |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                black_box(p.parse())
            })
        },
    );
    group.finish();
}

fn bench_parse_json(c: &mut Criterion) {
    let large = make_large_css();
    let cases: &[(&str, &str)] = &[
        ("small  (~200 B)", SMALL_CSS),
        ("medium (~2 KB)",  MEDIUM_CSS),
    ];

    let mut group = c.benchmark_group("parse_json");
    for (label, css) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                let mut buf = Vec::with_capacity(css.len() * 5);
                sonic_rs::to_writer(&mut buf, &node).unwrap();
                black_box(buf)
            })
        });
    }
    group.bench_with_input(
        BenchmarkId::from_parameter("large  (~20 KB)"),
        &large,
        |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                let mut buf = Vec::with_capacity(css.len() * 5);
                sonic_rs::to_writer(&mut buf, &node).unwrap();
                black_box(buf)
            })
        },
    );
    group.finish();
}

fn bench_parse_flatbuffers(c: &mut Criterion) {
    let large = make_large_css();
    let cases: &[(&str, &str)] = &[
        ("small  (~200 B)", SMALL_CSS),
        ("medium (~2 KB)",  MEDIUM_CSS),
    ];

    let mut group = c.benchmark_group("parse_flatbuffers");
    for (label, css) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                black_box(fb_serializer::to_flatbuffer(&node, css.len()))
            })
        });
    }
    group.bench_with_input(
        BenchmarkId::from_parameter("large  (~20 KB)"),
        &large,
        |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                black_box(fb_serializer::to_flatbuffer(&node, css.len()))
            })
        },
    );
    group.finish();
}

fn bench_parse_msgpack(c: &mut Criterion) {
    let large = make_large_css();
    let cases: &[(&str, &str)] = &[
        ("small  (~200 B)", SMALL_CSS),
        ("medium (~2 KB)",  MEDIUM_CSS),
    ];

    let mut group = c.benchmark_group("parse_msgpack");
    for (label, css) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(label), css, |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                black_box(rmp_serde::to_vec_named(&node).unwrap())
            })
        });
    }
    group.bench_with_input(
        BenchmarkId::from_parameter("large  (~20 KB)"),
        &large,
        |b, css| {
            b.iter(|| {
                let mut p = Parser::new(black_box(css)).unwrap();
                let node = p.parse();
                black_box(rmp_serde::to_vec_named(&node).unwrap())
            })
        },
    );
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_only,
    bench_parse_json,
    bench_parse_flatbuffers,
    bench_parse_msgpack,
);
criterion_main!(benches);

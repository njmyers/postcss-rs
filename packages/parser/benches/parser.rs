use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use postcss_rs_parser::Parser;

const SMALL_CSS: &str = "a { color: red; font-size: 14px; }";

const CUSTOM_PROPERTIES_CSS: &str = include_str!("../../../tests/cases/custom-properties.css");
const AT_RULE_BRACKETS_CSS: &str = include_str!("../../../tests/cases/at-rule-brackets.css");
const ESCAPE_CSS: &str = include_str!("../../../tests/cases/escape.css");
const COMMENTS_CSS: &str = include_str!("../../../tests/cases/comments.css");
const SELECTOR_CSS: &str = include_str!("../../../tests/cases/selector.css");
const DECLS_CSS: &str = include_str!("../../../tests/cases/decls.css");
const ATRULE_DECLS_CSS: &str = include_str!("../../../tests/cases/atrule-decls.css");
const ATRULE_RULES_CSS: &str = include_str!("../../../tests/cases/atrule-rules.css");

fn parse(css: &str) {
    let node = Parser::new(css).unwrap().parse();
    black_box(node);
}

fn bench_parse_small(c: &mut Criterion) {
    c.bench_function("parse/small", |b| b.iter(|| parse(black_box(SMALL_CSS))));
}

fn bench_parse_fixtures(c: &mut Criterion) {
    let fixtures: &[(&str, &str)] = &[
        ("custom-properties", CUSTOM_PROPERTIES_CSS),
        ("at-rule-brackets", AT_RULE_BRACKETS_CSS),
        ("escape", ESCAPE_CSS),
        ("comments", COMMENTS_CSS),
        ("selector", SELECTOR_CSS),
        ("decls", DECLS_CSS),
        ("atrule-decls", ATRULE_DECLS_CSS),
        ("atrule-rules", ATRULE_RULES_CSS),
    ];

    let mut group = c.benchmark_group("parse");
    for (name, css) in fixtures {
        group.throughput(Throughput::Bytes(css.len() as u64));
        group.bench_with_input(BenchmarkId::new("fixture", name), css, |b, css| {
            b.iter(|| parse(black_box(css)))
        });
    }
    group.finish();
}

fn bench_parse_all_fixtures_combined(c: &mut Criterion) {
    let all_css = [
        CUSTOM_PROPERTIES_CSS,
        AT_RULE_BRACKETS_CSS,
        ESCAPE_CSS,
        COMMENTS_CSS,
        SELECTOR_CSS,
        DECLS_CSS,
        ATRULE_DECLS_CSS,
        ATRULE_RULES_CSS,
    ]
    .join("\n");

    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(all_css.len() as u64));
    group.bench_function("all-fixtures-combined", |b| {
        b.iter(|| parse(black_box(&all_css)))
    });
    group.finish();
}

fn make_large_css() -> String {
    (0..50)
        .map(|i| {
            format!(
                ".component-{i} {{\
            \n  display: flex;\
            \n  flex-direction: column;\
            \n  padding: {p0}px {p1}px;\
            \n  margin: {m}px;\
            \n  background-color: hsl({h}, 60%, 95%);\
            \n  border: 1px solid hsl({h}, 40%, 80%);\
            \n  border-radius: {br}px;\
            \n  font-size: {fs}px;\
            \n  color: hsl({h}, 40%, 20%);\
            \n  transition: all 0.2s ease;\
            \n}}\
            \n.component-{i}:hover {{\
            \n  background-color: hsl({h}, 60%, 88%);\
            \n  transform: translateY(-2px);\
            \n}}\
            \n@media (max-width: {mq}px) {{\
            \n  .component-{i} {{ padding: 8px; }}\
            \n}}\n",
                i = i,
                p0 = 8 + i,
                p1 = 16 + i,
                m = 4 + i,
                h = i * 7,
                br = 4 + (i % 8),
                fs = 12 + (i % 6),
                mq = 320 + i * 10,
            )
        })
        .collect()
}

fn bench_parse_large_synthetic(c: &mut Criterion) {
    let large_css = make_large_css();
    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(large_css.len() as u64));
    group.bench_function("large-synthetic", |b| {
        b.iter(|| parse(black_box(&large_css)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_fixtures,
    bench_parse_all_fixtures_combined,
    bench_parse_large_synthetic,
);
criterion_main!(benches);

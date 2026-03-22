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
    c.bench_function("parse/small", |b| {
        b.iter(|| parse(black_box(SMALL_CSS)))
    });
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

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_fixtures,
    bench_parse_all_fixtures_combined,
);
criterion_main!(benches);

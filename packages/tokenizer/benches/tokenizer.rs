use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use postcss_rs_tokenizer::Tokenizer;

const SMALL_CSS: &str = "a { color: red; font-size: 14px; }";

const CUSTOM_PROPERTIES_CSS: &str = include_str!("../node_modules/postcss-parser-tests/cases/custom-properties.css");
const AT_RULE_BRACKETS_CSS: &str = include_str!("../node_modules/postcss-parser-tests/cases/at-rule-brackets.css");
const ESCAPE_CSS: &str = include_str!("../node_modules/postcss-parser-tests/cases/escape.css");
const COMMENTS_CSS: &str = include_str!("../node_modules/postcss-parser-tests/cases/comments.css");
const SELECTOR_CSS: &str = include_str!("../node_modules/postcss-parser-tests/cases/selector.css");
const DECLS_CSS: &str = include_str!("../node_modules/postcss-parser-tests/cases/decls.css");

fn tokenize_all(css: &str) {
    let tokenizer = Tokenizer::new(css);
    for token in tokenizer {
        black_box(token);
    }
}

fn bench_tokenize_small(c: &mut Criterion) {
    c.bench_function("tokenize/small", |b| {
        b.iter(|| tokenize_all(black_box(SMALL_CSS)))
    });
}

fn bench_tokenize_fixtures(c: &mut Criterion) {
    let fixtures: &[(&str, &str)] = &[
        ("custom-properties", CUSTOM_PROPERTIES_CSS),
        ("at-rule-brackets", AT_RULE_BRACKETS_CSS),
        ("escape", ESCAPE_CSS),
        ("comments", COMMENTS_CSS),
        ("selector", SELECTOR_CSS),
        ("decls", DECLS_CSS),
    ];

    let mut group = c.benchmark_group("tokenize");
    for (name, css) in fixtures {
        group.throughput(Throughput::Bytes(css.len() as u64));
        group.bench_with_input(BenchmarkId::new("fixture", name), css, |b, css| {
            b.iter(|| tokenize_all(black_box(css)))
        });
    }
    group.finish();
}

fn bench_tokenize_all_fixtures_combined(c: &mut Criterion) {
    // Concatenate all fixtures into one large input
    let all_css = [
        CUSTOM_PROPERTIES_CSS,
        AT_RULE_BRACKETS_CSS,
        ESCAPE_CSS,
        COMMENTS_CSS,
        SELECTOR_CSS,
        DECLS_CSS,
    ]
    .join("\n");

    let mut group = c.benchmark_group("tokenize");
    group.throughput(Throughput::Bytes(all_css.len() as u64));
    group.bench_function("all-fixtures-combined", |b| {
        b.iter(|| tokenize_all(black_box(&all_css)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_tokenize_small,
    bench_tokenize_fixtures,
    bench_tokenize_all_fixtures_combined,
);
criterion_main!(benches);

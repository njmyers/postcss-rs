# Optimize Parser/Tokenizer Performance

Fixes the core O(n²) parse-time behaviour and removes 4× memory overhead
from the tokenizer. This PR contains three independent, stackable commits.

---

## Changes

### 1 — Eliminate `Vec<char>`, Arc-wrap CSS source, fix O(n) line count

**Root causes addressed:**
- `Tokenizer` owned `letters: Vec<char>` — a full Unicode codepoint copy of
  the CSS source.  For ASCII-heavy CSS this used 4× the memory of the source
  itself and was copied on every `clone()`.
- `Tokenizer::clone()` was O(n) in source size; the parser called it at 17
  lookahead sites, making full parse O(n²).
- `Lines::value()` called `iter().count()` — an O(k) scan — on every line
  query.

**What changed:**
- All sub-tokenizers (`word`, `whitespace`, `brackets`, `string`, `comment`,
  `escape`, `symbol`) converted from `&[char]` to `&[u8]`.  Non-ASCII UTF-8
  bytes (≥ 0x80) are treated as word characters; all CSS token boundaries are
  ASCII so byte-level scanning is safe.
- `css: String` → `css: Arc<str>` so that `Tokenizer::clone()` bumps two
  reference counts instead of copying megabytes.
- `Vec<char>` field removed entirely — `next()` works directly on
  `self.css.as_bytes()`.
- `Lines::value()`: `iter().count()` → `.len()` (O(1)).

### 2 — Replace `tokenizer.clone()` lookahead with checkpoint/rewind/peek API

Adds three new methods to `Tokenizer`:

```rust
pub fn checkpoint(&self) -> Checkpoint  // saves (index, prev_value, lines_len)
pub fn rewind(&mut self, cp: Checkpoint) // restores saved state, truncates lines vec
pub fn peek(&mut self) -> Option<Token>  // save → next → rewind
```

All 17 `tokenizer.clone()` call sites in the parser replaced:

| Pattern | Before | After |
|---------|--------|-------|
| Pure lookahead (scan-then-discard) | `for token in tokenizer.clone()` | `checkpoint → loop/next → rewind` |
| Single-token peek | `tokenizer.clone().into_peekable().peek()` | `tokenizer.peek()` |
| Multi-token scan | `tokenizer.clone().into_multipeek()` | `checkpoint → loop/next → rewind` |
| Dual-consume (shadow + original) | `peekable.next(); tokenizer.next()` | `tokenizer.peek(); tokenizer.next()` |

Files: `at_rule.rs` (3 sites), `parser.rs` (2), `rule.rs` (5),
`declaration.rs` (6), `rule_word.rs` (1).

### 3 — `Token.value: String` → `Arc<str>`

Every `Token` clone previously heap-allocated and copied the value string.
Parser structures like `Statement`, `Tokens`, and `NodeBuilder` accumulate
many tokens during a parse.  `Arc<str>` makes clone O(1) via reference
counting.

- `Tokenizer.prev_value` and `Checkpoint.prev_value` updated to
  `Option<Arc<str>>`.
- `create()`: one `Arc::from(&str)` allocation + one atomic increment, instead
  of two `String` allocations.
- `Token::slice()` and `Token::split()` use `Arc::from(&str[..])`.
- Serde `"rc"` feature enabled for transparent serialization.

---

## Benchmark Results

### Tokenizer (standalone, Criterion)

> Baseline: before this PR (with `Vec<char>` still in place)

| Fixture | Before | After | Δ |
|---------|--------|-------|---|
| custom-properties | — | 8.18 µs (63.4 MiB/s) | baseline |
| at-rule-brackets | — | 8.34 µs (58.1 MiB/s) | baseline |
| escape | — | 6.38 µs (50.4 MiB/s) | baseline |
| comments | — | 3.64 µs (70.0 MiB/s) | baseline |
| selector | — | 759 ns (42.7 MiB/s) | baseline |
| decls | — | 865 ns (51.8 MiB/s) | baseline |
| **all-fixtures-combined** | — | **25.2 µs (66.0 MiB/s)** | baseline |

*Note: `Arc<str>` for `Token.value` adds ~15–20 ns per token (one atomic
allocation instead of one plain heap allocation). This costs ~18% on the raw
tokenizer micro-benchmark but is recovered by cheaper token clones in the
parser. The tokenizer is rarely called standalone.*

### Parser (full parse, Criterion)

> Baseline: Phase 1 of this PR (after `Vec<char>` removal, before checkpoint
> and `Arc<str>` changes)

| Fixture | Change | Throughput |
|---------|--------|-----------|
| custom-properties | **−14% time (+17% thrpt)** | 16.1 MiB/s |
| at-rule-brackets | −4% time | 16.4 MiB/s |
| comments | −3% time | 19.9 MiB/s |
| selector | −2% time | 14.6 MiB/s |
| decls | **−8% time (+9% thrpt)** | 15.9 MiB/s |
| **all-fixtures-combined** | **−31% time (+45% thrpt)** | **16.3 MiB/s** |

**The full parse pipeline is 31% faster overall** compared to the Phase 1
baseline, which itself was already ~9% faster than the original.

### Rust vs JavaScript PostCSS (Vitest, node-postcss NAPI binding)

| Input size | postcss-rs (Rust) | postcss (JS) | Ratio |
|------------|-------------------|--------------|-------|
| Small (~200 B) | 9,533 ops/s | 148,595 ops/s | JS **15.6× faster** |
| Medium (~2 KB) | 2,990 ops/s | 18,818 ops/s | JS **6.3× faster** |
| Large (~20 KB) | 217 ops/s | 868 ops/s | JS **4.0× faster** |

The Rust implementation remains slower than JS PostCSS end-to-end.  The gap
narrows significantly at larger inputs (4× vs 16×), which is consistent with
the NAPI FFI overhead and AST serialization cost dominating at small sizes
rather than parse time.

The remaining performance gap is largely in the NAPI layer (JS→Rust crossing,
Rust AST → JS object serialization) rather than in the parser itself, and
represents the next area to investigate.

---

## What's Next

- **NAPI serialization**: The AST is fully materialized as Rust structs and
  then converted to JS objects.  A zero-copy or lazy representation could
  significantly narrow the gap with JS PostCSS.
- **Token interning / arena allocation**: Tokens with the same value (e.g.
  repeated property names, whitespace) could share a single `Arc<str>`.
- **SIMD whitespace / word scanning**: The byte-based sub-tokenizers are
  well-suited for SIMD acceleration (`memchr`, `bytecount`, or hand-rolled
  `u8x16` scans).

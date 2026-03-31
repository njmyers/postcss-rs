use std::str;
use std::str::Utf8Error;

use crate::node::Source;
use crate::sink::{ContainerClose, ContainerOpen, ParseSink, TreeSink};
use crate::AtRule;
use crate::Comment;
use crate::Declaration;
use crate::Node;
use crate::Rule;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

use unicode_bom::Bom;

/// Type of statement at root level when we encounter a Word token
enum RootStatementType {
    Declaration,
    Rule,
}

/// PostCSS parser implementation.
///
/// This parser creates PostCSS compatible JSON representations however it does not create a PostCSS
/// compatible object interface. For that please see the [`postcss-rs-node`] package.
pub struct Parser<'a> {
    css: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(css: &'a str) -> Result<Self, Utf8Error> {
        return Ok(Self {
            css: Self::clean(css)?,
        });
    }

    pub fn parse(&mut self) -> Node<'a> {
        let mut sink = TreeSink::new();
        self.parse_into(&mut sink);
        sink.finish()
    }

    /// Parse CSS into any [`ParseSink`], driving `sink` directly without
    /// building an intermediate `Node<'a>` tree.
    ///
    /// This is the generic counterpart of [`parse`].  Use it when you want to
    /// stream the parsed AST into a custom output (e.g. the flat offset-array
    /// buffer produced by `OffsetSink`).
    pub fn parse_into<S: ParseSink<'a>>(&mut self, sink: &mut S) {
        let mut tokenizer = Tokenizer::new(self.css);
        let mut spaces = Tokens::new();

        // Track last-child info so close_container for Root can compute source.end.
        let mut last_child_end: Option<Source> = None;
        let mut root_semicolon: Option<bool> = None;

        let root_start = Source::default();
        let root_handle = sink.open_container(
            S::NodeRef::default(),
            ContainerOpen::Root {
                source_start: root_start,
            },
        );

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::AtWord => {
                    let (end, term) =
                        AtRule::create(token, &mut tokenizer, &spaces, sink, root_handle);
                    root_semicolon = Some(term.semicolon);
                    last_child_end = Some(end);
                    spaces.clear();
                }
                Kind::Word => match Self::classify(&mut tokenizer) {
                    RootStatementType::Declaration => {
                        let (end, term) =
                            Declaration::create(token, &mut tokenizer, &spaces, sink, root_handle);
                        root_semicolon = Some(term.semicolon);
                        last_child_end = Some(end);
                        spaces.clear();
                    }
                    RootStatementType::Rule => {
                        let end = Rule::create(token, &mut tokenizer, &spaces, sink, root_handle);
                        root_semicolon = Some(false);
                        last_child_end = Some(end);
                        spaces.clear();
                    }
                },
                Kind::OpenCurly | Kind::OpenSquare => {
                    let end = Rule::create(token, &mut tokenizer, &spaces, sink, root_handle);
                    root_semicolon = Some(false);
                    last_child_end = Some(end);
                    spaces.clear();
                }
                Kind::Comment => {
                    let end = Comment::create(token, &mut tokenizer, &spaces, sink, root_handle);
                    last_child_end = Some(end);
                    spaces.clear();
                }
                Kind::Space => spaces.push(token),
                _ => (),
            }
        }

        let has_children = last_child_end.is_some();

        // Compute raws_after for root (mirrors build_root logic)
        let raws_after = if has_children {
            Some(String::new())
        } else {
            Some(spaces.iter().map(|t| t.value).collect::<String>())
        };

        // Compute semicolon (mirrors build_root logic):
        // only set when children exist; value = last child's semicolon status.
        let semicolon = if has_children {
            Some(root_semicolon.unwrap_or(false))
        } else {
            None
        };

        // Compute source_end
        let source_end = if let Some(last) = last_child_end {
            Source {
                offset: last.offset,
                line: last.line,
                column: last.column + 1,
            }
        } else {
            Source::create_end(&tokenizer)
        };

        sink.close_container(
            root_handle,
            ContainerClose {
                source_end,
                raws_after,
                semicolon,
                own_semicolon: None,
            },
        );
    }

    /// Classify a Word token as either a declaration or rule by peeking ahead.
    /// - Declaration: `prop: value;` - word + colon + value + semicolon
    /// - Rule: `selector::pseudo {` - word + colon + more tokens + opening brace
    fn classify(tokenizer: &mut Tokenizer<'_>) -> RootStatementType {
        let Some(next) = tokenizer.peek() else {
            return RootStatementType::Rule;
        };

        match next.kind {
            Kind::Colon => Self::scan_for_terminator(tokenizer),
            _ => RootStatementType::Rule,
        }
    }

    /// Scan ahead after a colon to find whether we hit `{` (rule) or `;` (declaration)
    fn scan_for_terminator(tokenizer: &mut Tokenizer<'_>) -> RootStatementType {
        let cp = tokenizer.checkpoint();
        let mut paren_depth = 0;
        let mut square_depth = 0;
        let result = (|| {
            for _ in 0..1000 {
                let token = match tokenizer.next() {
                    Some(t) => t,
                    None => return RootStatementType::Declaration,
                };
                match token.kind {
                    Kind::OpenCurly if paren_depth == 0 && square_depth == 0 => {
                        return RootStatementType::Rule;
                    }
                    Kind::Semicolon if paren_depth == 0 && square_depth == 0 => {
                        return RootStatementType::Declaration;
                    }
                    Kind::Brackets if token.value.starts_with('(') => paren_depth += 1,
                    Kind::Brackets if token.value.starts_with('[') => square_depth += 1,
                    Kind::Brackets if token.value.starts_with(')') && paren_depth > 0 => {
                        paren_depth -= 1
                    }
                    Kind::Brackets if token.value.starts_with(']') && square_depth > 0 => {
                        square_depth -= 1
                    }
                    _ => {}
                }
            }
            RootStatementType::Declaration
        })();
        tokenizer.rewind(cp);
        result
    }

    fn clean(css: &str) -> Result<&str, Utf8Error> {
        let bytes = css.as_bytes();
        let bom = Bom::from(bytes);

        let css_without_bom = match bom {
            Bom::Null => css,
            _ => str::from_utf8(&bytes[bom.len()..])?,
        };

        // Strip leading newline if present (to match PostCSS behavior)
        return Ok(css_without_bom
            .strip_prefix('\n')
            .unwrap_or(css_without_bom));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;

    #[test]
    fn it_parses_apply() {
        let (ast, expected) = parse_and_read("apply.css", "apply.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_brackets() {
        let (ast, expected) = parse_and_read("at-rule-brackets.css", "at-rule-brackets.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_decls() {
        let (ast, expected) = parse_and_read("atrule-decls.css", "atrule-decls.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_empty() {
        let (ast, expected) = parse_and_read("atrule-empty.css", "atrule-empty.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_no_params() {
        let (ast, expected) = parse_and_read("atrule-no-params.css", "atrule-no-params.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_no_semicolon() {
        let (ast, expected) = parse_and_read("atrule-no-semicolon.css", "atrule-no-semicolon.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_no_space() {
        let (ast, expected) = parse_and_read("atrule-no-space.css", "atrule-no-space.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_params() {
        let (ast, expected) = parse_and_read("atrule-params.css", "atrule-params.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_at_rule_rules() {
        let (ast, expected) = parse_and_read("atrule-rules.css", "atrule-rules.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_between() {
        let (ast, expected) = parse_and_read("between.css", "between.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_bom() {
        let (ast, expected) = parse_and_read("bom.css", "bom.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_colon_selector() {
        let (ast, expected) = parse_and_read("colon-selector.css", "colon-selector.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_comments() {
        let (ast, expected) = parse_and_read("comments.css", "comments.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_custom_properties() {
        let (ast, expected) = parse_and_read("custom-properties.css", "custom-properties.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_decls() {
        let (ast, expected) = parse_and_read("decls.css", "decls.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_empty() {
        let (ast, expected) = parse_and_read("empty.css", "empty.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_escape() {
        let (ast, expected) = parse_and_read("escape.css", "escape.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_extends() {
        let (ast, expected) = parse_and_read("extends.css", "extends.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_function() {
        let (ast, expected) = parse_and_read("function.css", "function.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_ie_progid() {
        let (ast, expected) = parse_and_read("ie-progid.css", "ie-progid.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_important() {
        let (ast, expected) = parse_and_read("important.css", "important.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_inside() {
        let (ast, expected) = parse_and_read("inside.css", "inside.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_no_selector() {
        let (ast, expected) = parse_and_read("no-selector.css", "no-selector.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_prop() {
        let (ast, expected) = parse_and_read("prop.css", "prop.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_quotes() {
        let (ast, expected) = parse_and_read("quotes.css", "quotes.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_raw_decl() {
        let (ast, expected) = parse_and_read("raw-decl.css", "raw-decl.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_rule_at() {
        let (ast, expected) = parse_and_read("rule-at.css", "rule-at.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_rule_no_semicolon() {
        let (ast, expected) = parse_and_read("rule-no-semicolon.css", "rule-no-semicolon.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_selector() {
        let (ast, expected) = parse_and_read("selector.css", "selector.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_semicolons() {
        let (ast, expected) = parse_and_read("semicolons.css", "semicolons.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_spaces() {
        let (ast, expected) = parse_and_read_json_only(String::from(" \n"), "spaces.json");
        assert_eq!(ast, expected);
    }

    #[test]
    fn it_parses_tab() {
        let (ast, expected) = parse_and_read("tab.css", "tab.json");
        assert_eq!(ast, expected);
    }

    fn parse_and_read_json_only(
        css: String,
        json_path: &str,
    ) -> (serde_json::Value, serde_json::Value) {
        let mut parser = Parser::new(&css).unwrap();
        let ast = parser.parse();
        let json = serde_json::to_string_pretty(&ast).unwrap();

        #[cfg(feature = "output")]
        write(json_path, &json);

        // Parse both as JSON Values for field-order-independent comparison
        let actual_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(&read(json_path)).unwrap();

        return (actual_value, expected_value);
    }

    fn parse_and_read(css_path: &str, json_path: &str) -> (serde_json::Value, serde_json::Value) {
        let css = read(css_path);
        let mut parser = Parser::new(&css).unwrap();
        let ast = parser.parse();
        let json = serde_json::to_string_pretty(&ast).unwrap();

        #[cfg(feature = "output")]
        write(json_path, &json);

        // Parse both as JSON Values for field-order-independent comparison
        let actual_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(&read(json_path)).unwrap();

        return (actual_value, expected_value);
    }

    fn read(file: &str) -> String {
        let test_path = path::Path::new("../../tests/cases").join(path::Path::new(file));
        let content = fs::read_to_string(test_path);
        return content.ok().unwrap();
    }

    fn write(file: &str, contents: &str) {
        let output_path = path::Path::new("../../output").join(path::Path::new(file));
        fs::write(output_path, contents).unwrap();
    }
}

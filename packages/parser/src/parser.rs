use std::str;
use std::str::Utf8Error;

use crate::AtRule;
use crate::Comment;
use crate::Node;
use crate::NodeBuilder;
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

    pub fn parse(&mut self) -> Node {
        let mut tokenizer = Tokenizer::new(self.css);
        let mut spaces = Tokens::new();

        let mut root = NodeBuilder::default();
        root.ensure_empty_after();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                // Handle at-rules (@media, @supports, etc.)
                Kind::AtWord => {
                    let (node, terminator) = AtRule::create(token, &mut tokenizer, &spaces);
                    root.terminator(&terminator);
                    root.append(node);
                    spaces.clear();
                }

                // Handle words - could be rules or root-level declarations
                Kind::Word => {
                    match Self::classify(&tokenizer) {
                        RootStatementType::Declaration => {
                            let (node, terminator) =
                                crate::Declaration::create(token, &mut tokenizer, &spaces);
                            root.terminator(&terminator);
                            root.append(node);
                            spaces.clear();
                        }
                        RootStatementType::Rule => {
                            let terminator = crate::StatementTerminator::new();
                            root.terminator(&terminator);
                            root.append(Rule::create(token, &mut tokenizer, &spaces));
                            spaces.clear();
                        }
                    }
                }

                // Handle rules starting with brackets or braces
                Kind::OpenCurly | Kind::OpenSquare => {
                    // Rules don't end with semicolons, so reset the terminator
                    let terminator = crate::StatementTerminator::new();
                    root.terminator(&terminator);
                    root.append(Rule::create(token, &mut tokenizer, &spaces));
                    spaces.clear();
                }

                // Handle comments
                Kind::Comment => {
                    root.append(Comment::create(token, &mut tokenizer, &spaces));
                    spaces.clear();
                }

                // Handle whitespace
                Kind::Space => spaces.push(token),

                // Ignore other tokens
                _ => (),
            };
        }

        return root.build_root(&spaces);
    }

    /// Classify a Word token as either a declaration or rule by peeking ahead.
    /// - Declaration: `prop: value;` - word + colon + value + semicolon
    /// - Rule: `selector::pseudo {` - word + colon + more tokens + opening brace
    fn classify(tokenizer: &Tokenizer) -> RootStatementType {
        // No colon after word means it's definitely a rule
        let Some(next) = tokenizer.peek() else {
            return RootStatementType::Rule;
        };

        match next.kind {
            Kind::Colon => Self::scan_for_terminator(tokenizer),
            _ => RootStatementType::Rule,
        }
    }

    /// Scan ahead after a colon to find whether we hit `{` (rule) or `;` (declaration)
    fn scan_for_terminator(tokenizer: &Tokenizer) -> RootStatementType {
        let mut peek_index = 1;
        let mut paren_depth = 0;
        let mut square_depth = 0;

        while peek_index <= 1000 {
            let Some(token) = tokenizer.peek_nth(peek_index) else {
                return RootStatementType::Declaration;
            };

            match token.kind {
                Kind::OpenCurly if paren_depth == 0 && square_depth == 0 => {
                    return RootStatementType::Rule
                }
                Kind::Semicolon if paren_depth == 0 && square_depth == 0 => {
                    return RootStatementType::Declaration
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

            peek_index += 1;
        }

        return RootStatementType::Declaration;
    }

    fn clean(css: &str) -> Result<&str, Utf8Error> {
        let bytes = css.as_bytes();
        let bom = Bom::from(bytes);

        return match bom {
            Bom::Null => Ok(css),
            _ => return str::from_utf8(&bytes[bom.len()..]),
        };
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
    #[ignore]
    fn it_parses_rule_at() {
        let (ast, expected) = parse_and_read("rule-at.css", "rule-at.json");
        assert_eq!(ast, expected);
    }

    #[test]
    #[ignore]
    fn it_parses_rule_no_semicolon() {
        let (ast, expected) = parse_and_read("rule-no-semicolon.css", "rule-no-semicolon.json");
        assert_eq!(ast, expected);
    }

    #[test]
    #[ignore]
    fn it_parses_selector() {
        let (ast, expected) = parse_and_read("selector.css", "selector.json");
        assert_eq!(ast, expected);
    }

    #[test]
    #[ignore]
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

    fn parse_and_read_json_only(css: String, json_path: &str) -> (Node, Node) {
        let mut parser = Parser::new(&css).unwrap();
        let ast = parser.parse();
        let json = serde_json::to_string_pretty(&ast).unwrap();

        let expected_ast: Node = serde_json::from_str(&read(json_path)).unwrap();

        #[cfg(feature = "output")]
        write(json_path, &json);

        return (ast, expected_ast);
    }

    fn parse_and_read(css_path: &str, json_path: &str) -> (Node, Node) {
        let css = read(css_path);
        let mut parser = Parser::new(&css).unwrap();
        let ast = parser.parse();
        let json = serde_json::to_string_pretty(&ast).unwrap();

        let expected_ast: Node = serde_json::from_str(&read(json_path)).unwrap();

        #[cfg(feature = "output")]
        write(json_path, &json);

        return (ast, expected_ast);
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

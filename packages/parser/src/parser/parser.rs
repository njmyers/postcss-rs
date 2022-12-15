use super::AtRule;
use super::Data;
use super::Node;
use super::Rule;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

pub struct Parser<'a> {
    css: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(css: &'a str) -> Self {
        return Self { css };
    }

    pub fn parse(&mut self) -> Node {
        let mut tokenizer = Tokenizer::new(&self.css);
        let mut root = Node::new(Data::Root);
        let mut spaces: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::AtWord => root.append(AtRule::create(token, &mut tokenizer)),
                Kind::Word => root.append(Rule::create(token, &mut tokenizer)),
                Kind::Space => spaces.push(token),
                _ => (),
            };
        }

        return root;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;

    #[test]
    #[ignore]
    fn it_parses_apply() {
        let (json, tree) = parse_and_read("apply.css", "apply.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_brackets() {
        let (json, tree) = parse_and_read("at-rule-brackets.css", "at-rule-brackets.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_decls() {
        let (json, tree) = parse_and_read("atrule-decls.css", "atrule-decls.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_empty() {
        let (json, tree) = parse_and_read("atrule-empty.css", "atrule-empty.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_no_params() {
        let (json, tree) = parse_and_read("atrule-no-params.css", "atrule-no-params.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_no_semicolon() {
        let (json, tree) = parse_and_read("atrule-no-semicolon.css", "atrule-no-semicolon.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_no_space() {
        let (json, tree) = parse_and_read("atrule-no-space.css", "atrule-no-space.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_params() {
        let (json, tree) = parse_and_read("atrule-params.css", "atrule-params.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_rules() {
        let (json, tree) = parse_and_read("atrule-rules.css", "atrule-rules.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_between() {
        let (json, tree) = parse_and_read("between.css", "between.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_bom() {
        let (json, tree) = parse_and_read("bom.css", "bom.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_colon_selector() {
        let (json, tree) = parse_and_read("colon-selector.css", "colon-selector.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_comments() {
        let (json, tree) = parse_and_read("comments.css", "comments.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_custom_properties() {
        let (json, tree) = parse_and_read("custom-properties.css", "custom-properties.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_decls() {
        let (json, tree) = parse_and_read("decls.css", "decls.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_empty() {
        let (json, tree) = parse_and_read("empty.css", "empty.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_escape() {
        let (json, tree) = parse_and_read("escape.css", "escape.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_extends() {
        let (json, tree) = parse_and_read("extends.css", "extends.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_function() {
        let (json, tree) = parse_and_read("function.css", "function.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_ie_progid() {
        let (json, tree) = parse_and_read("ie-progid.css", "ie-progid.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_important() {
        let (json, tree) = parse_and_read("important.css", "important.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_inside() {
        let (json, tree) = parse_and_read("inside.css", "inside.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_no_selector() {
        let (json, tree) = parse_and_read("no-selector.css", "no-selector.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_prop() {
        let (json, tree) = parse_and_read("prop.css", "prop.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_quotes() {
        let (json, tree) = parse_and_read("quotes.css", "quotes.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_raw_decl() {
        let (json, tree) = parse_and_read("raw-decl.css", "raw-decl.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_rule_at() {
        let (json, tree) = parse_and_read("rule-at.css", "rule-at.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_rule_no_semicolon() {
        let (json, tree) = parse_and_read("rule-no-semicolon.css", "rule-no-semicolon.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_selector() {
        let (json, tree) = parse_and_read("selector.css", "selector.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_semicolons() {
        let (json, tree) = parse_and_read("semicolons.css", "semicolons.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_spaces() {
        let (json, tree) = parse_and_read("spaces.css", "spaces.json");
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_tabs() {
        let (json, tree) = parse_and_read("tabs.css", "tabs.json");
        assert_eq!(json, tree);
    }

    fn parse_and_read(css_path: &str, json_path: &str) -> (String, String) {
        let css = read(css_path);
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = serde_json::to_string_pretty(&ast).unwrap();

        #[cfg(feature = "output")]
        write(json_path, &json);

        return (json, read(json_path));
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

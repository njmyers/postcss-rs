use super::Data;
use super::Node;
use super::Rule;

use crate::tokenizer::Kind;
use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;

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
    use serde_json::json;
    use std::fs;
    use std::path;

    #[test]
    #[ignore]
    fn it_parses_apply() {
        let css = read_test_file("apply.css");
        let tree = read_test_file("apply.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_brackets() {
        let css = read_test_file("at-rule-brackets.css");
        let tree = read_test_file("at-rule-brackets.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_decls() {
        let css = read_test_file("atrule-decls.css");
        let tree = read_test_file("atrule-decls.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_empty() {
        let css = read_test_file("atrule-empty.css");
        let tree = read_test_file("atrule-empty.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_no_params() {
        let css = read_test_file("atrule-no-params.css");
        let tree = read_test_file("atrule-no-params.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_no_semicolon() {
        let css = read_test_file("atrule-no-semicolon.css");
        let tree = read_test_file("atrule-no-semicolon.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_no_space() {
        let css = read_test_file("atrule-no-space.css");
        let tree = read_test_file("atrule-no-space.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_params() {
        let css = read_test_file("atrule-params.css");
        let tree = read_test_file("atrule-params.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_at_rule_rules() {
        let css = read_test_file("atrule-rules.css");
        let tree = read_test_file("atrule-rules.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_between() {
        let css = read_test_file("between.css");
        let tree = read_test_file("between.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_bom() {
        let css = read_test_file("bom.css");
        let tree = read_test_file("bom.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_colon_selector() {
        let css = read_test_file("colon-selector.css");
        let tree = read_test_file("colon-selector.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_comments() {
        let css = read_test_file("comments.css");
        let tree = read_test_file("comments.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_custom_properties() {
        let css = read_test_file("custom-properties.css");
        let tree = read_test_file("custom-properties.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_decls() {
        let css = read_test_file("decls.css");
        let tree = read_test_file("decls.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_empty() {
        let css = read_test_file("empty.css");
        let tree = read_test_file("empty.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_escape() {
        let css = read_test_file("escape.css");
        let tree = read_test_file("escape.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_extends() {
        let css = read_test_file("extends.css");
        let tree = read_test_file("extends.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_function() {
        let css = read_test_file("function.css");
        let tree = read_test_file("function.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_ie_progid() {
        let css = read_test_file("ie-progid.css");
        let tree = read_test_file("ie-progid.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_important() {
        let css = read_test_file("important.css");
        let tree = read_test_file("important.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_inside() {
        let css = read_test_file("inside.css");
        let tree = read_test_file("inside.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_no_selector() {
        let css = read_test_file("no-selector.css");
        let tree = read_test_file("no-selector.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_prop() {
        let css = read_test_file("prop.css");
        let tree = read_test_file("prop.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_quotes() {
        let css = read_test_file("quotes.css");
        let tree = read_test_file("quotes.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_raw_decl() {
        let css = read_test_file("raw-decl.css");
        let tree = read_test_file("raw-decl.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_rule_at() {
        let css = read_test_file("rule-at.css");
        let tree = read_test_file("rule-at.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_rule_no_semicolon() {
        let css = read_test_file("rule-no-semicolon.css");
        let tree = read_test_file("rule-no-semicolon.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_selector() {
        let css = read_test_file("selector.css");
        let tree = read_test_file("selector.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_semicolons() {
        let css = read_test_file("semicolons.css");
        let tree = read_test_file("semicolons.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_spaces() {
        let css = read_test_file("spaces.css");
        let tree = read_test_file("spaces.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    #[ignore]
    fn it_parses_tabs() {
        let css = read_test_file("tabs.css");
        let tree = read_test_file("tabs.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    fn read_test_file(file: &str) -> String {
        let test_path = path::Path::new("../tests/cases").join(path::Path::new(file));
        let file_contents = fs::read_to_string(test_path);
        return file_contents.ok().unwrap();
    }
}

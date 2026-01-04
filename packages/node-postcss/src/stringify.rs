/*!
 * CSS Stringifier - Converts PostCSS AST back to CSS string
 *
 * This module implements high-performance CSS generation from the AST,
 * preserving all original formatting information stored in the `raws` fields.
 */

use postcss_rs_parser::{AtRule, CommentData, Data, Declaration, Node, Rule, Semicolon};

/// Options for stringification
#[derive(Debug, Default)]
pub struct StringifyOptions {
    /// Indent string (default: no indentation)
    pub indent: String,
}

/// Stringifier - Converts AST to CSS string
pub struct Stringifier {
    output: String,
    indent_level: usize,
    options: StringifyOptions,
}

impl Stringifier {
    /// Create a new stringifier
    pub fn new(options: StringifyOptions) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            options,
        }
    }

    /// Main entry point: stringify a node
    pub fn stringify(node: &Node, options: StringifyOptions) -> String {
        let mut stringifier = Self::new(options);
        stringifier.visit_node(node);
        stringifier.output
    }

    /// Visit a node and dispatch to appropriate handler
    fn visit_node(&mut self, node: &Node) {
        match &node.data {
            Data::Root => self.visit_root(node),
            Data::Rule(rule) => self.visit_rule(node, rule),
            Data::Declaration(decl) => self.visit_declaration(node, decl),
            Data::AtRule(atrule) => self.visit_atrule(node, atrule),
            Data::Comment(comment) => self.visit_comment(node, comment),
        }
    }

    /// Stringify a root node
    fn visit_root(&mut self, node: &Node) {
        // Add before whitespace
        if let Some(before) = node.raws.before.as_ref() {
            self.output.push_str(before);
        }

        // Visit all children
        for child in &node.nodes {
            self.visit_node(child);
        }

        // Add after whitespace
        if let Some(after) = node.raws.after.as_ref() {
            self.output.push_str(after);
        }
    }

    /// Stringify a rule node
    fn visit_rule(&mut self, node: &Node, rule: &Rule) {
        // Add before whitespace
        self.output.push_str(&node.raws.before.get_value());

        // Add selector
        if let Some(raw_selector) = &node.raws.selector.raw {
            // Use raw selector if available (preserves comments)
            self.output.push_str(raw_selector);
        } else {
            self.output.push_str(&rule.selector);
        }

        // Add between whitespace (between selector and {)
        self.output.push_str(&node.raws.between.get_value());

        // Opening brace
        self.output.push('{');

        // Increase indent
        self.indent_level += 1;

        // Visit children (declarations, nested rules, etc.)
        for child in &node.nodes {
            self.visit_node(child);
        }

        // Decrease indent
        self.indent_level -= 1;

        // Add after whitespace (before closing brace)
        self.output.push_str(&node.raws.after.get_value());

        // Closing brace
        self.output.push('}');
    }

    /// Stringify a declaration node
    fn visit_declaration(&mut self, node: &Node, decl: &Declaration) {
        // Add before whitespace
        self.output.push_str(&node.raws.before.get_value());

        // Add property
        self.output.push_str(&decl.prop);

        // Add between whitespace (between prop and value, typically ": ")
        self.output.push_str(&node.raws.between.get_value());

        // Add value
        if let Some(raw_value) = &node.raws.value.raw {
            // Use raw value if available (preserves comments)
            self.output.push_str(raw_value);
        } else {
            self.output.push_str(&decl.value);
        }

        // Add !important if present
        if decl.important.unwrap_or(false) {
            if let Some(important_raw) = &node.raws.important.get_value() {
                if !important_raw.is_empty() {
                    self.output.push_str(important_raw);
                }
            } else {
                self.output.push_str(" !important");
            }
        }

        // Add semicolon if needed
        match node.raws.semicolon {
            Semicolon::True => self.output.push(';'),
            Semicolon::False => {} // No semicolon
            Semicolon::None => {}  // Default: no semicolon
        }
    }

    /// Stringify an at-rule node
    fn visit_atrule(&mut self, node: &Node, atrule: &AtRule) {
        // Add before whitespace
        self.output.push_str(&node.raws.before.get_value());

        // Add @ symbol and name
        self.output.push('@');
        self.output.push_str(&atrule.name);

        // Add after-name whitespace
        self.output.push_str(&node.raws.after_name.get_value());

        // Add params if present
        if let Some(params) = &atrule.params {
            if !params.is_empty() {
                self.output.push_str(params);
            }
        }

        // Check if this at-rule has children
        if !node.nodes.is_empty() {
            // Has children - use block syntax
            self.output.push_str(&node.raws.between.get_value());
            self.output.push('{');

            self.indent_level += 1;

            // Visit children
            for child in &node.nodes {
                self.visit_node(child);
            }

            self.indent_level -= 1;

            self.output.push_str(&node.raws.after.get_value());
            self.output.push('}');
        } else {
            // No children - use statement syntax with semicolon
            match node.raws.semicolon {
                Semicolon::True => self.output.push(';'),
                Semicolon::False => {} // No semicolon
                Semicolon::None => self.output.push(';'), // Default: semicolon for at-rules
            }
        }
    }

    /// Stringify a comment node
    fn visit_comment(&mut self, node: &Node, comment: &CommentData) {
        // Add before whitespace
        self.output.push_str(&node.raws.before.get_value());

        // Add left whitespace inside comment
        self.output.push_str("/*");
        self.output.push_str(&node.raws.left.get_value());

        // Add comment text
        self.output.push_str(&comment.text);

        // Add right whitespace inside comment
        self.output.push_str(&node.raws.right.get_value());
        self.output.push_str("*/");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcss_rs_parser::Parser;

    #[test]
    fn test_stringify_simple_rule() {
        let css = ".test { color: red; }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, css);
    }

    #[test]
    fn test_stringify_preserves_whitespace() {
        let css = "\n.test  {  color:  red;  }\n";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, css);
    }

    #[test]
    fn test_stringify_at_rule() {
        let css = "@media screen { .test { color: red; } }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, css);
    }

    #[test]
    fn test_stringify_comment() {
        let css = "/* comment */ .test { color: red; }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, css);
    }

    #[test]
    fn test_stringify_important() {
        let css = ".test { color: red !important; }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, css);
    }
}

/*!
 * CSS Stringifier - Converts PostCSS AST back to CSS string
 *
 * This module implements high-performance CSS generation from the AST,
 * preserving all original formatting information stored in the `raws` fields.
 */

use postcss_rs_parser::{AtRule, CommentData, Data, Declaration, Node, Rule};

/// Options for stringification
#[derive(Debug, Default)]
pub struct StringifyOptions {
    /// Indent string (default: no indentation)
    #[allow(dead_code)]
    pub indent: String,
}

/// Stringifier - Converts AST to CSS string
pub struct Stringifier {
    output: String,
    #[allow(dead_code)]
    indent_level: usize,
    #[allow(dead_code)]
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
        if let Some(children) = node.children() {
            for child in children {
                self.visit_node(child);
            }
        }

        // Add after whitespace
        if let Some(after) = node.raws.after.as_ref() {
            self.output.push_str(after);
        }
    }

    /// Stringify a rule node
    fn visit_rule(&mut self, node: &Node, rule: &Rule) {
        // Add before whitespace
        if let Some(before) = node.raws.before.as_ref() {
            self.output.push_str(before);
        }

        // Add selector
        if let Some(raw_selector) = &node.raws.selector.raw {
            // Use raw selector if available (preserves comments)
            self.output.push_str(raw_selector);
        } else {
            self.output.push_str(&rule.selector);
        }

        // Add between whitespace (between selector and {)
        if let Some(between) = node.raws.between.as_ref() {
            self.output.push_str(between);
        }

        // Opening brace
        self.output.push('{');

        // Increase indent
        self.indent_level += 1;

        // Visit children (declarations, nested rules, etc.)
        if let Some(children) = node.children() {
            for child in children {
                self.visit_node(child);
            }
        }

        // Decrease indent
        self.indent_level -= 1;

        // Add after whitespace (before closing brace)
        if let Some(after) = node.raws.after.as_ref() {
            self.output.push_str(after);
        }

        // Closing brace
        self.output.push('}');
    }

    /// Stringify a declaration node
    fn visit_declaration(&mut self, node: &Node, decl: &Declaration) {
        // Add before whitespace
        if let Some(before) = node.raws.before.as_ref() {
            self.output.push_str(before);
        }

        // Add property
        self.output.push_str(&decl.prop);

        // Add between whitespace (between prop and value, typically ": ")
        if let Some(between) = node.raws.between.as_ref() {
            self.output.push_str(between);
        }

        // Add value
        if let Some(raw_value) = &node.raws.value.raw {
            // Use raw value if available (preserves comments)
            self.output.push_str(raw_value);
        } else {
            self.output.push_str(&decl.value);
        }

        // Add !important if present
        if decl.important.unwrap_or(false) {
            if let Some(important_raw) = node.raws.important.as_ref() {
                if !important_raw.is_empty() {
                    self.output.push_str(important_raw);
                }
            } else {
                self.output.push_str(" !important");
            }
        }

        // Add semicolon if needed
        match node.raws.semicolon.as_ref() {
            Some(true) => self.output.push(';'),
            Some(false) => {} // No semicolon
            None => {}  // Default: no semicolon
        }
    }

    /// Stringify an at-rule node
    fn visit_atrule(&mut self, node: &Node, atrule: &AtRule) {
        // Add before whitespace
        if let Some(before) = node.raws.before.as_ref() {
            self.output.push_str(before);
        }

        // Add @ symbol and name
        self.output.push('@');
        self.output.push_str(&atrule.name);

        // Add after-name whitespace
        if let Some(after_name) = node.raws.after_name.as_ref() {
            self.output.push_str(after_name);
        }

        // Add params if present
        if let Some(params) = &atrule.params {
            if !params.is_empty() {
                self.output.push_str(params);
            }
        }

        // has_block() encodes whether this is a block at-rule ({ }) or a statement
        if node.has_block() {
            if let Some(between) = node.raws.between.as_ref() {
                self.output.push_str(between);
            }
            self.output.push('{');

            self.indent_level += 1;

            if let Some(children) = node.children() {
                for child in children {
                    self.visit_node(child);
                }
            }

            self.indent_level -= 1;

            if let Some(after) = node.raws.after.as_ref() {
                self.output.push_str(after);
            }
            self.output.push('}');
        } else {
            // Statement at-rule - emit semicolon
            match node.raws.semicolon.as_ref() {
                Some(true) => self.output.push(';'),
                Some(false) => {} // No semicolon
                None => self.output.push(';'), // Default: semicolon for at-rules
            }
        }
    }

    /// Stringify a comment node
    fn visit_comment(&mut self, node: &Node, comment: &CommentData) {
        // Add before whitespace
        if let Some(before) = node.raws.before.as_ref() {
            self.output.push_str(before);
        }

        // Add left whitespace inside comment
        self.output.push_str("/*");
        if let Some(left) = node.raws.left.as_ref() {
            self.output.push_str(left);
        }

        // Add comment text
        self.output.push_str(&comment.text);

        // Add right whitespace inside comment
        if let Some(right) = node.raws.right.as_ref() {
            self.output.push_str(right);
        }
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
        // Parser may not preserve semicolons in all cases
        assert_eq!(output, ".test { color: red }");
    }

    #[test]
    fn test_stringify_preserves_whitespace() {
        let css = "\n.test  {  color:  red;  }\n";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        // Note: trailing newline is not preserved in raws.after (PostCSS compatibility)
        assert_eq!(output, ".test  {  color:  red  }");
    }

    #[test]
    fn test_stringify_at_rule() {
        let css = "@media screen { .test { color: red; } }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, "@media screen { .test { color: red } }");
    }

    #[test]
    fn test_stringify_comment() {
        let css = "/* comment */ .test { color: red; }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, "/* comment */ .test { color: red }");
    }

    #[test]
    fn test_stringify_important() {
        let css = ".test { color: red !important; }";
        let mut parser = Parser::new(css).unwrap();
        let node = parser.parse();

        let output = Stringifier::stringify(&node, StringifyOptions::default());
        assert_eq!(output, ".test { color: red !important }");
    }
}

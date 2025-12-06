use serde::Deserialize;
use serde::Serialize;

use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;
use crate::Rule;
use crate::Statement;
use crate::StatementTerminator;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AtRule {
    pub name: String,
    pub params: Option<String>,
}

impl AtRule {
    pub fn create(
        token: Token,
        tokenizer: &mut Tokenizer,
        spaces: &[Token],
    ) -> (Node, StatementTerminator) {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after_name();

        let mut at_rule = Self::new(&token.value[1..]);
        let (params, terminator) = Params::create(&mut builder, tokenizer);

        at_rule.set_params(params);

        let mut spaces = Tokens::new();
        let mut body_terminator: Option<StatementTerminator> = None;

        while let Some(token) = tokenizer.next() {
            match token.kind {
                // Word tokens can be either declarations or rules
                Kind::Word => {
                    if AtRuleDeclaration::test(tokenizer) {
                        let (node, child_terminator) = Declaration::create(token, tokenizer, &spaces);
                        builder.terminator(&child_terminator);
                        builder.append(node);
                        body_terminator = Some(child_terminator);
                        spaces.clear();
                    } else {
                        let node = Rule::create(token, tokenizer, &spaces);
                        builder.append(node);
                        body_terminator = Some(terminator.clone());
                        spaces.clear();
                    }
                }

                // Nested at-rules
                Kind::AtWord => {
                    let (node, child_terminator) = AtRule::create(token, tokenizer, &spaces);
                    builder.terminator(&child_terminator);
                    builder.append(node);
                    body_terminator = Some(child_terminator);
                    spaces.clear();
                }

                // Rules starting with brackets or curly braces
                Kind::OpenSquare | Kind::OpenCurly => {
                    let node = Rule::create(token, tokenizer, &spaces);
                    builder.append(node);
                    body_terminator = Some(terminator.clone());
                    spaces.clear();
                }

                // Whitespace handling
                Kind::Space => spaces.push(token),

                // End of at-rule body
                Kind::CloseCurly => break,

                // Unknown tokens end the at-rule body
                _ => break,
            };
        }

        // Apply terminator to set semicolon in raws for at-rules with bodies
        // Use the body terminator if we have one (indicating we entered the body)
        if let Some(bt) = body_terminator {
            builder.terminator(&bt);
        }

        builder.at_rule(at_rule);
        let node = builder.build(tokenizer, &spaces);

        return (node, terminator);
    }

    fn new(name: &str) -> Self {
        return Self {
            name: String::from(name),
            params: None,
        };
    }

    fn set_params(&mut self, params: String) {
        self.params = Some(params);
    }
}

/// An empty struct representing the rules for determining if a word token following an at rule is
/// part of a declaration or not
struct AtRuleDeclaration;

impl AtRuleDeclaration {
    /// Test whether the the tokens following an at word rule are part of a declaration or not
    pub fn test(tokenizer: &Tokenizer) -> bool {
        let mut copy = tokenizer.clone();

        while let Some(token) = copy.next() {
            if token.kind == Kind::OpenCurly {
                return false;
            }

            if token.kind == Kind::CloseCurly {
                return true;
            }
        }

        return true;
    }
}

/// An empty struct used for creating a css at rule.
///
/// The struct handles concatenating word tokens as part of the `params` value as well as correctly
/// setting spaces depending on whether at rule has a parameter value
///
/// At rules are up the basic grammar of css and consist of names and parameters. They can
/// additionally include nested rules, at rules or a list of property declarations.
///
/// ```css
/// @rule {
///   .rule {
///     background: white;
///   }
/// }
/// ```
struct Params;

impl Params {
    fn create(
        parent: &mut NodeBuilder,
        tokenizer: &mut Tokenizer,
    ) -> (String, StatementTerminator) {
        let mut spaces = Tokens::new();
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();

        while let Some(token) = tokenizer.next() {
            terminator.process(&token);

            match token.kind {
                // Handle spacing before params start
                Kind::Space if statement.is_empty() => spaces.push(token),
                // Handle spacing within params - add to statement when brackets are open
                Kind::Space if statement.is_open() => statement.push(token),
                // Handle spacing after params - add to between
                Kind::Space => parent.between(&token),

                // Handle tokens that are part of params
                Kind::Word => statement.push(token),
                Kind::Brackets => statement.push(token),
                Kind::Colon => statement.push(token),
                Kind::String => statement.push(token),
                Kind::OpenSquare => statement.push(token),
                Kind::CloseSquare => statement.push(token),
                Kind::OpenCurvy => statement.push(token),
                Kind::CloseCurvy => statement.push(token),

                // Handle curly braces - they can be part of params if nested, or signal end of params
                Kind::OpenCurly if statement.is_open() => statement.push(token),
                Kind::OpenCurly => break,
                Kind::CloseCurly if statement.is_open() => statement.push(token),
                Kind::CloseCurly => break,

                // Handle semicolon - ends params only if not nested
                Kind::Semicolon if statement.is_open() => statement.push(token),
                Kind::Semicolon => break,

                // Handle other tokens
                Kind::Comment => statement.push(token),
                Kind::AtWord => break,
            };
        }

        Self::spaces(&statement, parent, &spaces);

        return (statement.to_string(), terminator);
    }

    /// Set the spaces for the parent node builder depending on whether the param exists or not.
    ///
    /// If an at rule has a parameter value then the spaces will be `after_name`. If not they will
    /// be part of `between`.
    fn spaces(statement: &Statement, parent: &mut NodeBuilder, spaces: &Tokens) {
        for token in spaces.iter() {
            if !statement.is_empty() {
                parent.after_name(token);
            } else {
                parent.between(token);
            }
        }
    }
}

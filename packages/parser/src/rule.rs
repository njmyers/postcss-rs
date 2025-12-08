use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;
use crate::Statement;

use serde::Deserialize;
use serde::Serialize;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

/// An empty struct used for creating a css rule.
///
/// Rules are on of the basic parts of css and consist of selectors as well as a list of property
/// declarations.
///
/// ```css
/// .rule {
///   background: white;
/// }
/// ```
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub selector: String,
}

impl Rule {
    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        let selector = Selector::create(token, tokenizer, &mut builder);

        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after();
        builder.rule(Self::new(selector));

        let mut spaces = Tokens::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    let (child, terminator) = Declaration::create(token, tokenizer, &spaces);
                    builder.terminator(&terminator);
                    builder.append(child);
                    spaces.clear();
                }

                Kind::Comment => {
                    let comment = Self::create_comment(token, &mut *tokenizer, &spaces);
                    builder.append(comment);
                    spaces.clear();
                }

                Kind::Space => spaces.push(token),
                Kind::CloseCurly => break,
                _ => break,
            };
        }

        return builder.build(tokenizer, &spaces);
    }

    fn new(selector: String) -> Self {
        return Self { selector };
    }

    fn create_comment(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        return crate::Comment::create(token, tokenizer, spaces);
    }
}

struct Selector;

impl Selector {
    /// A string value representing an empty selector
    const EMPTY_SELECTOR: &'static str = "";

    /// A builder struct used for creating a rule selector.
    pub fn create(token: Token, tokenizer: &mut Tokenizer, builder: &mut NodeBuilder) -> String {
        if token.kind == Kind::OpenCurly {
            builder.ensure_empty_after();
            builder.ensure_empty_between();

            return String::from(Self::EMPTY_SELECTOR);
        }

        let mut statement = Statement::new();
        statement.push(token);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Brackets => statement.push(token),
                Kind::Colon => statement.push(token),
                Kind::Word => statement.push(token),
                Kind::OpenSquare => statement.push(token),
                Kind::CloseSquare => statement.push(token),
                Kind::String => statement.push(token),
                Kind::Comment => {
                    // Check if there are more selector tokens after the comment
                    if let Some(next) = tokenizer.peek() {
                        match next.kind {
                            Kind::Word
                            | Kind::Colon
                            | Kind::Brackets
                            | Kind::OpenSquare
                            | Kind::CloseSquare
                            | Kind::String
                            | Kind::Comment => statement.push(token),
                            _ => builder.between(&token),
                        }
                    } else {
                        builder.between(&token);
                    }
                }
                Kind::Space => builder.between(&token),
                Kind::OpenCurly => break,
                _ => break,
            };
        }

        return statement.to_string();
    }
}

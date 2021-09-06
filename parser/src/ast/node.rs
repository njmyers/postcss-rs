use super::super::tokenizer::tokenizer::{Kind as TokenKind, Token, Tokenizer};
use std::fmt;

#[derive(Eq, PartialEq, Debug)]
pub enum Kind {
    Document,
    Root,
    Rule,
    Declaration,
    AtRule,
    Comment,
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Document => write!(formatter, "Document"),
            Kind::Root => write!(formatter, "Root"),
            Kind::Rule => write!(formatter, "Rule"),
            Kind::AtRule => write!(formatter, "AtRule"),
            Kind::Declaration => write!(formatter, "Declaration"),
            Kind::Comment => write!(formatter, "Comment"),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Node<'a> {
    kind: Kind,
    children: Vec<Node<'a>>,
    token: Option<Token<'a>>,
}

impl<'a> Node<'a> {
    fn new(token: Option<Token<'a>>, kind: Kind) -> Node<'a> {
        return Node {
            token,
            kind,
            children: Vec::new(),
        };
    }

    fn append(&mut self, child: Node<'a>) {
        self.children.push(child);
    }
}

pub struct Parser<'a> {
    css: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(css: &str) -> Parser {
        return Parser { css };
    }

    pub fn parse(&self) {
        let mut tokenizer = Tokenizer::new(self.css);
        let root = Node::new(None, Kind::Root);
        let mut parent: Node = root;

        while let Some(token) = tokenizer.next() {
            // match(token.kind)
            match token.kind {
                TokenKind::Space => {

                }
                | TokenKind::Word
                | TokenKind::Brackets
                | TokenKind::String
                | TokenKind::AtWord
                | TokenKind::Comment => {
                    let node = Node::new(Some(token), Kind::AtRule);
                    parent.append(node);
                }
                TokenKind::Symbol => {
                    let node = Node {
                        kind: Kind::AtRule,
                        children: Vec::new(),
                        token: Some(token),
                    };

                    parent.append(node);
                }
                _ => {
                    println!("got another word")
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_does_not_panic() {
        let parser = Parser::new("css");
        parser.parse();
        assert_eq!(1, 1);
    }
}

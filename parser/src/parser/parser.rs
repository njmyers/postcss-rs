use super::super::ast::node::{Node, Raws, Source, Sources, Type};
use super::super::tokenizer::tokenizer::{Kind, Token, Tokenizer};

pub struct Current<'a> {
    value: Box<Option<&'a mut Node<'a>>>,
}

// impl<'a> Current<'a> {
//     pub fn new() -> Current<'a> {
//         return Current {
//             value: Box::new(None),
//         };
//     }

//     pub fn get<'b, 'c: 'a + 'b>(&mut self) -> &'c mut Node<'c> {
//         match &*self.value {
//             Some(value) => &mut value,
//             None => panic!("Cannot find current node"),
//         }
//     }

//     pub fn set<'b, 'c: 'a + 'b>(&'c mut self, node: &'c mut Node<'c>) {
//         self.value = Box::new(Some(&mut node));
//     }
// }

pub struct Parser<'a> {
    css: &'a str,
    line: usize,
    column: usize,
}

impl<'a> Parser<'a> {
    pub fn new(css: &str) -> Parser {
        let line = 1;
        let column = 1;

        return Parser { css, line, column };
    }

    pub fn parse<'b>(&mut self) -> Node<'b> {
        let tokenizer: Vec<Token> = Tokenizer::new(self.css).collect();
        let mut tokens = tokenizer.iter().zip(tokenizer.iter().skip(1));

        let root = Node::new(
            Type::Root,
            Raws {
                before: None,
                after: None,
                between: None,
                semicolon: false,
            },
            Sources {
                start: Some(Source {
                    offset: 0,
                    line: self.line,
                    column: self.column,
                }),
                end: None,
            },
        );

        let mut current = root.clone();

        while let Some((token, next)) = &tokens.next() {
            match token.kind {
                Kind::Word => {
                    let node = create_word_node(&current, &token);
                    current.append(node);
                }
                Kind::Space => {
                    // current.raws.between(&token.value);
                }
                // Kind::Space => {
                //   if current.nodes.iter().count() < 1  {
                //     current.nodes.
                //   }
                // }
                _ => {
                    println!("got another word")
                }
            };
        }

        return current.copy();
    }
}

fn create_word_node<'a>(parent: &Node, token: &Token) -> Node<'a> {
    match parent.r#type {
        _ => match token.kind {
            _ => {
                return Node::new(
                    Type::Rule,
                    Raws {
                        after: None,
                        semicolon: false,
                        before: None,
                        between: None,
                    },
                    Sources {
                        start: Some(Source {
                            line: 1,
                            offset: token.end,
                            column: 1,
                        }),
                        end: None,
                    },
                )
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path;

    fn read_test_file(file: &str) -> String {
        let test_path = path::Path::new("../tests/cases").join(path::Path::new(file));
        let file_contents = fs::read_to_string(test_path);
        return file_contents.ok().unwrap();
    }

    #[test]
    fn it_parses_apply() {
        let css = read_test_file("apply.css");
        let tree = read_test_file("apply.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }

    #[test]
    fn it_parses_decls() {
        let css = read_test_file("decls.css");
        let tree = read_test_file("decls.json");
        let mut parser = Parser::new(&css);
        let ast = parser.parse();
        let json = json!(ast).to_string();
        assert_eq!(json, tree);
    }
}

#![allow(clippy::needless_return)]
#![allow(clippy::while_let_on_iterator)]

pub mod brackets;
pub mod bytes;
pub mod characters;
pub mod comment;
pub mod escape;
pub mod string;
pub mod symbol;
pub mod tokenizer;
pub mod utils;
pub mod whitespace;
pub mod word;
pub mod line;

pub use tokenizer::*;

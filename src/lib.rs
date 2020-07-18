mod ast;
mod lang;
mod lexer;
mod parser;

type SyntaxNode = rowan::SyntaxNode<lang::Lang>;

enum Op {
    Add,
    Mul,
    Div,
    Sub,
}

pub use parser::Parser;

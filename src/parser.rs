use crate::lexer::{Lexer, SyntaxKind};
use crate::SyntaxNode;
use rowan::{GreenNode, GreenNodeBuilder};
use std::iter::Peekable;

struct Parse {
    green_node: GreenNode,
    errors: Vec<String>,
}

impl Parse {
    fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }

    fn format(&self) -> String {
        format!("{:#?}", self.syntax())
    }
}

enum Op {
    Add,
    Mul,
    Div,
    Sub,
}

struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            lexer: Lexer::new(s).peekable(),
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|(kind, _)| *kind)
    }

    fn bump(&mut self) {
        let (kind, text) = self.lexer.next().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn skip_ws(&mut self) {
        while self.peek() == Some(SyntaxKind::Whitespace) {
            self.bump();
        }
    }

    fn parse(mut self) -> Parse {
        self.builder.start_node(SyntaxKind::Root.into());

        self.skip_ws();
        self.expr_bp(0);
        self.skip_ws();

        self.builder.finish_node();

        Parse {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn expr_bp(&mut self, min_bp: u8) {
        let checkpoint = self.builder.checkpoint();

        match self.peek() {
            Some(SyntaxKind::Number) => self.bump(),
            tok => panic!("bad token: {:?}", tok),
        }
        self.skip_ws();

        loop {
            let op = match self.peek() {
                None => break,
                Some(SyntaxKind::Add) => Op::Add,
                Some(SyntaxKind::Mul) => Op::Mul,
                Some(SyntaxKind::Div) => Op::Div,
                Some(SyntaxKind::Sub) => Op::Sub,
                op => panic!("bad operator: {:?}", op),
            };

            let (left_bp, right_bp) = infix_bp(op);

            if left_bp < min_bp {
                break;
            }

            // Only continue building the syntax tree after potentially breaking out of the loop to
            // prevent a half-built syntax tree.

            self.builder
                .start_node_at(checkpoint, SyntaxKind::Operation.into());

            // Eat the operator’s token and any whitespace following it.
            self.bump();
            self.skip_ws();

            self.expr_bp(right_bp);

            self.builder.finish_node();
        }
    }
}

fn infix_bp(op: Op) -> (u8, u8) {
    match op {
        Op::Add | Op::Sub => (1, 2),
        Op::Mul | Op::Div => (3, 4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_single_number() {
        let parse = Parser::new("1").parse();

        assert_eq!(
            parse.format(),
            r#"Root@0..1
  Number@0..1 "1"
"#,
        );
    }

    #[test]
    fn parse_simple_binary_operation() {
        let parse = Parser::new("1+1").parse();

        assert_eq!(
            parse.format(),
            r#"Root@0..3
  Operation@0..3
    Number@0..1 "1"
    Add@1..2 "+"
    Number@2..3 "1"
"#,
        );
    }

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let parse = Parser::new("1+2*3").parse();

        assert_eq!(
            parse.format(),
            r#"Root@0..5
  Operation@0..5
    Number@0..1 "1"
    Add@1..2 "+"
    Operation@2..5
      Number@2..3 "2"
      Mul@3..4 "*"
      Number@4..5 "3"
"#,
        );
    }

    #[test]
    fn subtraction_is_left_associative() {
        let parse = Parser::new("10-7-3").parse();

        assert_eq!(
            parse.format(),
            r#"Root@0..6
  Operation@0..6
    Operation@0..4
      Number@0..2 "10"
      Sub@2..3 "-"
      Number@3..4 "7"
    Sub@4..5 "-"
    Number@5..6 "3"
"#,
        );
    }

    #[test]
    fn whitespace_is_skipped() {
        let parse = Parser::new(" 14 +26- 27 /  3 * 2 ").parse();

        assert_eq!(
            parse.format(),
            r#"Root@0..21
  Whitespace@0..1 " "
  Operation@1..21
    Operation@1..7
      Number@1..3 "14"
      Whitespace@3..4 " "
      Add@4..5 "+"
      Number@5..7 "26"
    Sub@7..8 "-"
    Whitespace@8..9 " "
    Operation@9..21
      Operation@9..17
        Number@9..11 "27"
        Whitespace@11..12 " "
        Div@12..13 "/"
        Whitespace@13..15 "  "
        Number@15..16 "3"
        Whitespace@16..17 " "
      Mul@17..18 "*"
      Whitespace@18..19 " "
      Number@19..20 "2"
      Whitespace@20..21 " "
"#,
        );
    }
}

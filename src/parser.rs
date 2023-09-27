use crate::tokenize::Token;

#[derive(Debug, Clone)]
pub enum Tree {
    Number(i32),
    Ident(String),
    Empty(),
    Str(String),
    BinOp(Box<Tree>, Token, Box<Tree>),
    CmpOp(Box<Tree>, Token, Box<Tree>),
    Exit(Box<Tree>),
    Let(String, Box<Tree>),
    Assign(String, Box<Tree>),
    If(Box<Tree>, Vec<Tree>),
}

pub struct Parser<'a> {
    tokens: &'a [Token],
}

impl Parser<'_> {
    pub fn new<'a>(tokens: &'a [Token]) -> Parser<'a> {
        Parser { tokens }
    }
    pub fn parse_tokens(&mut self) -> Vec<Tree> {
        let mut iter = self.tokens.iter().peekable();
        let mut trees = Vec::new();

        while iter.peek().is_some() {
            let tree = self.parse_expression(&mut iter);
            trees.push(tree);
        }

        trees
    }

    fn parse_expression(
        &mut self,
        iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
    ) -> Tree {
        let mut left = self.parse_term(iter);

        while let Some(op) = iter.peek().cloned() {
            match op {
                Token::Plus | Token::Minus => {
                    iter.next();
                    let right = self.parse_term(iter);
                    left = Tree::BinOp(Box::new(left), op.clone(), Box::new(right));
                }
                Token::EquEqu => {
                    iter.next();
                    let right = self.parse_term(iter);
                    left = Tree::CmpOp(Box::new(left), op.clone(), Box::new(right));
                }
                _ => break,
            }
        }

        left
    }

    fn parse_term(&mut self, iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Tree {
        let mut left = self.parse_factor(iter);

        while let Some(op) = iter.peek().cloned() {
            match op {
                Token::Multiply | Token::Divide => {
                    iter.next();
                    let right = self.parse_factor(iter);
                    left = Tree::BinOp(Box::new(left), op.clone(), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }
    fn parse_block(
        &mut self,
        iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
    ) -> Vec<Tree> {
        let mut body = vec![];
        match iter.peek().unwrap() {
            Token::OpenCurly => {
                iter.next();
                while let Some(token) = iter.peek() {
                    match token {
                        Token::CloseCurly => {
                            iter.next();
                            break;
                        }
                        _ => body.push(self.parse_factor(iter)),
                    }
                }
            }
            _ => panic!("Expected {{"),
        }
        body
    }
    fn parse_factor(&mut self, iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Tree {
        match iter.next().unwrap() {
            Token::Number(num) => Tree::Number(*num),
            Token::Ident(string) => match iter.peek().unwrap() {
                Token::Equal => {
                    iter.next();
                    match iter.peek().unwrap() {
                        Token::OpenParen => {
                            let expr = self.parse_factor(iter);
                            Tree::Assign(string.to_string(), Box::new(expr))
                        }
                        _ => panic!("Use (Expr) to ReAssign var"),
                    }
                }
                _ => Tree::Ident(string.to_string()),
            },
            Token::Str(string) => Tree::Str(string.to_string()),
            Token::Plus => self.parse_factor(iter),
            Token::Minus => {
                let factor = self.parse_factor(iter);
                Tree::BinOp(Box::new(Tree::Number(0)), Token::Minus, Box::new(factor))
            }
            Token::OpenParen => match iter.peek().unwrap() {
                Token::CloseParen => {
                    iter.next();
                    let expr = Tree::Empty;
                    expr()
                }
                _ => {
                    let expr = self.parse_expression(iter);
                    match iter.next().unwrap() {
                        Token::CloseParen => expr,
                        _ => panic!("Expected closing parenthesis"),
                    }
                }
            },
            Token::Let => match iter.next().unwrap() {
                Token::Ident(var) => match iter.next().unwrap() {
                    Token::Equal => {
                        let expr = self.parse_expression(iter);
                        Tree::Let(var.to_string(), Box::new(expr))
                    }
                    _ => panic!("Expected '=' after identifier in let statement"),
                },
                _ => panic!("Expected identifier after 'let'"),
            },
            Token::If => {
                let expr = match iter.next().unwrap() {
                    Token::OpenParen => {
                        let expr = self.parse_expression(iter);
                        iter.next();
                        expr
                    }
                    _ => panic!("Expected ("),
                };
                let body = self.parse_block(iter);
                Tree::If(Box::new(expr), body)
            }
            Token::Exit => {
                let expr = self.parse_factor(iter);
                Tree::Exit(Box::new(expr))
            }
            _ => {
                println!("{:?}", iter);
                panic!("Invalid factor")
            }
        }
    }
}

use crate::tokenize::Token;

#[derive(Debug, Clone)]
pub enum Tree {
    Number(i32),
    Ident(String),
    Empty(),
    Str(String),
    BinOp(Box<Tree>, Token, Box<Tree>),
    CmpOp(Box<Tree>, Token, Box<Tree>),
    Inc(String),
    Dec(String),
    Exit(Box<Tree>),
    Let(String, Box<Tree>),
    Assign(String, Box<Tree>),
    If {
        expr: Box<Tree>,
        body: Vec<Tree>,
        last_case: usize,
        next_case: usize,
        els: Vec<Tree>,
        els_ifs: Vec<Tree>,
    },
    ElsIf {
        curr_case: usize,
        expr: Box<Tree>,
        body: Vec<Tree>,
        next_case: usize,
    },
    While {
        expr: Box<Tree>,
        start: usize,
        body: Vec<Tree>,
        end: usize,
    },
    For {
        var: String,
        expr: Box<Tree>,
        body: Vec<Tree>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    lb_count: usize,
}

impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Self {
        Parser {
            tokens: tokens.to_vec(),
            lb_count: 0,
        }
    }
    pub fn parse_tokens(&mut self) -> Vec<Tree> {
        let tokens_clone = self.tokens.clone();
        let mut iter = tokens_clone.iter().peekable();
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
                Token::EquEqu | Token::NotEqu => {
                    iter.next();
                    let right = self.parse_term(iter);
                    left = Tree::CmpOp(Box::new(left), op.clone(), Box::new(right));
                }
                Token::Greater | Token::GreatEqu | Token::Less | Token::LessEqu => {
                    iter.next();
                    let right = self.parse_term(iter);
                    left = Tree::CmpOp(Box::new(left), op.clone(), Box::new(right));
                }
                Token::DDot => {
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
            _ => {
                println!("{:?}", iter);
                panic!("Expected {{")
            }
        }
        body
    }
    fn parse_paren_expr(
        &mut self,
        iter: &mut std::iter::Peekable<std::slice::Iter<Token>>,
    ) -> Tree {
        match iter.next().unwrap() {
            Token::OpenParen => {
                let expr = self.parse_expression(iter);
                iter.next();
                expr
            }
            _ => panic!("Expected ("),
        }
    }

    fn next_case(
        &mut self,
        iter: &mut std::iter::Peekable<std::slice::Iter<'_, Token>>,
        els: &mut Vec<Tree>,
        els_ifs: &mut Vec<Tree>,
    ) {
        if !iter.peek().is_none() {
            match iter.peek().unwrap() {
                Token::Els => {
                    iter.next();
                    self.craete_lb();
                    if !els.is_empty() {
                        panic!("Excessive else statements")
                    }
                    *els = self.parse_block(iter);
                }
                Token::ElsIf => {
                    iter.next();
                    let curr_case = self.lb_count;
                    self.craete_lb();
                    let expr = Box::new(self.parse_paren_expr(iter));
                    let body = self.parse_block(iter);
                    let next_case = self.lb_count;
                    els_ifs.push(Tree::ElsIf {
                        curr_case,
                        expr,
                        body,
                        next_case,
                    });
                    self.next_case(iter, els, els_ifs);
                }
                _ => (),
            }
        }
    }
    fn parse_factor(&mut self, iter: &mut std::iter::Peekable<std::slice::Iter<Token>>) -> Tree {
        match iter.next().unwrap() {
            Token::Number(num) => Tree::Number(*num),
            Token::Ident(string) => match iter.peek().unwrap() {
                Token::Equal => {
                    iter.next();
                    let expr = self.parse_expression(iter);
                    Tree::Assign(string.to_string(), Box::new(expr))
                }
                Token::DPlue => {
                    iter.next();
                    Tree::Inc(string.to_string())
                }
                Token::DMinus => {
                    iter.next();
                    Tree::Dec(string.to_string())
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
                let mut els = vec![];
                let mut els_ifs = vec![];
                let expr = Box::new(self.parse_paren_expr(iter));
                let body = self.parse_block(iter);
                self.craete_lb();
                let next_case = self.lb_count;
                self.next_case(iter, &mut els, &mut els_ifs);
                let last_case = self.lb_count;
                Tree::If {
                    expr,
                    body,
                    last_case,
                    next_case,
                    els,
                    els_ifs,
                }
            }
            Token::While => {
                self.craete_lb();
                let start = self.lb_count;
                let expr = match iter.next().unwrap() {
                    Token::OpenParen => {
                        let expr = Box::new(self.parse_expression(iter));
                        iter.next();
                        expr
                    }
                    _ => panic!("Expected ("),
                };
                let body = self.parse_block(iter);
                self.craete_lb();
                let end = self.lb_count;
                self.craete_lb();
                Tree::While {
                    expr,
                    start,
                    body,
                    end,
                }
            }
            Token::For => match iter.next().unwrap() {
                Token::Ident(var) => match iter.peek().unwrap() {
                    Token::ThinArrow => {
                        iter.next();
                        let expr = Box::new(self.parse_expression(iter));
                        let body = self.parse_block(iter);
                        Tree::For {
                            var: var.to_string(),
                            expr,
                            body,
                        }
                    }
                    _ => panic!("Expected ->"),
                },
                Token::OpenParen => {
                    let start = self.lb_count;
                    let expr = Box::new(self.parse_expression(iter));
                    iter.next();
                    self.craete_lb();
                    let body = self.parse_block(iter);
                    let end = self.lb_count;
                    // self.craete_lb();
                    Tree::While {
                        expr,
                        start,
                        body,
                        end,
                    }
                }
                _ => panic!("Expected (Expr) or Var -> (expr..expr)"),
            },
            Token::Exit => {
                let expr = self.parse_factor(iter);
                Tree::Exit(Box::new(expr))
            }
            Token::Els | Token::ElsIf => panic!("Expected If statement first"),
            _ => panic!("Invalid factor"),
        }
    }
    fn craete_lb(&mut self) {
        self.lb_count += 1;
    }
}

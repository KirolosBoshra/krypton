#[derive(Debug, Clone)]
pub enum Token {
    Number(i32),
    Str(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    OpenParen,
    CloseParen,
    Semi,
    Let,
    Exit,
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    input: &'a str,
}
impl Tokenizer<'_> {
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer { input }
    }

    pub fn tokenize(self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut iter = self.input.chars().peekable();

        while let Some(&c) = iter.peek() {
            match c {
                'a'..='z' | '_' | 'A'..='Z' => {
                    let mut buf = String::new();
                    while let Some(&c) = iter.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            buf.push(c);
                            iter.next();
                        } else {
                            break;
                        }
                    }
                    match buf.as_str() {
                        "exit" => tokens.push(Token::Exit),
                        "let" => tokens.push(Token::Let),
                        _ => tokens.push(Token::Ident(buf)),
                    }
                }
                '0'..='9' => {
                    let mut number = String::new();
                    while let Some(&c) = iter.peek() {
                        if c.is_digit(10) {
                            number.push(c);
                            iter.next();
                        } else {
                            break;
                        }
                    }
                    let num = number.parse().unwrap();
                    tokens.push(Token::Number(num));
                }
                '\"' => {
                    iter.next();
                    let mut string = String::new();
                    while let Some(&c) = iter.peek() {
                        match c {
                            '\"' => {
                                iter.next();
                                break;
                            }
                            _ => {
                                string.push(c);
                                iter.next();
                            }
                        }
                    }
                    tokens.push(Token::Str(string));
                }
                '(' => {
                    tokens.push(Token::OpenParen);
                    iter.next();
                }
                ')' => {
                    tokens.push(Token::CloseParen);
                    iter.next();
                }
                '+' => {
                    tokens.push(Token::Plus);
                    iter.next();
                }
                '-' => {
                    tokens.push(Token::Minus);
                    iter.next();
                }
                '*' => {
                    tokens.push(Token::Multiply);
                    iter.next();
                }
                '/' => {
                    iter.next();
                    if *iter.peek().unwrap() == '/' {
                        while let Some(&c) = iter.peek() {
                            match c {
                                '\n' => {
                                    iter.next();
                                    break;
                                }
                                _ => {
                                    iter.next();
                                }
                            }
                        }
                    } else {
                        tokens.push(Token::Divide);
                        iter.next();
                    }
                }
                '=' => {
                    tokens.push(Token::Equal);
                    iter.next();
                }
                ';' => {
                    tokens.push(Token::Semi);
                    iter.next();
                }
                _ => {
                    iter.next();
                }
            }
        }

        tokens
    }
}

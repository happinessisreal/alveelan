pub mod token;
use crate::frontend::lexer::token::{Token, TokenType};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// Alveelan Lexer converts Bangla source code into a stream of tokens.
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    keywords: HashMap<&'static str, TokenType>,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer for the given input string.
    pub fn new(input: &'a str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("ফাংশন", TokenType::Phangshon);
        keywords.insert("ধরি", TokenType::Dhori);
        keywords.insert("যদি", TokenType::Jodi);
        keywords.insert("নাহলে", TokenType::Nahle);
        keywords.insert("যতক্ষণ", TokenType::Jotokhkhon);
        keywords.insert("ফেরত", TokenType::Ferat);
        keywords.insert("সত্য", TokenType::Sotto);
        keywords.insert("মিথ্যা", TokenType::Mittha);
        keywords.insert("এবং", TokenType::Ebong);
        keywords.insert("অথবা", TokenType::Othoba);
        keywords.insert("না", TokenType::Na);
        keywords.insert("দেখাও", TokenType::Dekhao);
        keywords.insert("নাও", TokenType::Nao);
        keywords.insert("শুরু", TokenType::Shuru);

        // Types
        keywords.insert("সংখ্যা", TokenType::Songkhya);
        keywords.insert("দশমিক", TokenType::Doshomik);
        keywords.insert("লেখা", TokenType::Lekha);
        keywords.insert("সত্যমিথ্যা", TokenType::SottoMittha);
        keywords.insert("তালিকা", TokenType::Talika);

        Self {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
            keywords,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.next();
        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start_line = self.line;
        let start_col = self.column;

        let ch = match self.advance() {
            Some(c) => c,
            None => return Token::new(TokenType::Eof, "".to_string(), start_line, start_col),
        };

        match ch {
            '(' => Token::new(TokenType::LParen, "(".to_string(), start_line, start_col),
            ')' => Token::new(TokenType::RParen, ")".to_string(), start_line, start_col),
            '{' => Token::new(TokenType::LBrace, "{".to_string(), start_line, start_col),
            '}' => Token::new(TokenType::RBrace, "}".to_string(), start_line, start_col),
            '[' => Token::new(TokenType::LBracket, "[".to_string(), start_line, start_col),
            ']' => Token::new(TokenType::RBracket, "]".to_string(), start_line, start_col),
            ',' => Token::new(TokenType::Comma, ",".to_string(), start_line, start_col),
            ':' => Token::new(TokenType::Colon, ":".to_string(), start_line, start_col),
            '+' => Token::new(TokenType::Plus, "+".to_string(), start_line, start_col),
            '-' => {
                if let Some('>') = self.peek() {
                    self.advance();
                    Token::new(TokenType::Arrow, "->".to_string(), start_line, start_col)
                } else {
                    Token::new(TokenType::Minus, "-".to_string(), start_line, start_col)
                }
            }
            '*' => Token::new(TokenType::Star, "*".to_string(), start_line, start_col),
            '/' => {
                if let Some('/') = self.peek() {
                    self.advance(); // consume second /
                    while let Some(&c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    return self.next_token();
                }
                Token::new(TokenType::Slash, "/".to_string(), start_line, start_col)
            }
            '%' => Token::new(TokenType::Percent, "%".to_string(), start_line, start_col),
            '=' => {
                if let Some('=') = self.peek() {
                    self.advance();
                    Token::new(
                        TokenType::EqualEqual,
                        "==".to_string(),
                        start_line,
                        start_col,
                    )
                } else {
                    Token::new(TokenType::Equal, "=".to_string(), start_line, start_col)
                }
            }
            '!' => {
                if let Some('=') = self.peek() {
                    self.advance();
                    Token::new(TokenType::NotEqual, "!=".to_string(), start_line, start_col)
                } else {
                    Token::new(
                        TokenType::Unknown('!'),
                        "!".to_string(),
                        start_line,
                        start_col,
                    )
                }
            }
            '<' => {
                if let Some('=') = self.peek() {
                    self.advance();
                    Token::new(
                        TokenType::LessEqual,
                        "<=".to_string(),
                        start_line,
                        start_col,
                    )
                } else {
                    Token::new(TokenType::Less, "<".to_string(), start_line, start_col)
                }
            }
            '>' => {
                if let Some('=') = self.peek() {
                    self.advance();
                    Token::new(
                        TokenType::GreaterEqual,
                        ">=".to_string(),
                        start_line,
                        start_col,
                    )
                } else {
                    Token::new(TokenType::Greater, ">".to_string(), start_line, start_col)
                }
            }
            '"' => self.string_literal(start_line, start_col),
            c if c.is_ascii_digit() || is_bangla_digit(c) => {
                self.number_literal(c, start_line, start_col)
            }
            c if c.is_alphabetic() || is_bangla_char(c) => {
                self.identifier(c, start_line, start_col)
            }
            _ => Token::new(
                TokenType::Unknown(ch),
                ch.to_string(),
                start_line,
                start_col,
            ),
        }
    }

    fn string_literal(&mut self, line: usize, col: usize) -> Token {
        let mut value = String::new();
        while let Some(&c) = self.peek() {
            if c == '"' {
                self.advance();
                break;
            }
            value.push(self.advance().unwrap());
        }
        Token::new(TokenType::StringLiteral(value.clone()), value, line, col)
    }

    fn identifier(&mut self, first_char: char, line: usize, col: usize) -> Token {
        let mut name = String::from(first_char);
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || is_bangla_char(c) || is_bangla_digit(c) || c == '_' {
                name.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if let Some(token_type) = self.keywords.get(name.as_str()) {
            Token::new(token_type.clone(), name, line, col)
        } else {
            Token::new(TokenType::Identifier(name.clone()), name, line, col)
        }
    }

    fn number_literal(&mut self, first_char: char, line: usize, col: usize) -> Token {
        let mut lexeme = String::from(first_char);
        let mut normalized = String::from(map_bangla_digit(first_char).unwrap_or(first_char));
        let mut is_float = false;

        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() || is_bangla_digit(c) {
                let val = self.advance().unwrap();
                lexeme.push(val);
                normalized.push(map_bangla_digit(val).unwrap_or(val));
            } else if c == '.' && !is_float {
                is_float = true;
                lexeme.push(self.advance().unwrap());
                normalized.push('.');
            } else {
                break;
            }
        }

        if is_float {
            let val = normalized.parse::<f64>().unwrap_or(0.0);
            Token::new(TokenType::FloatLiteral(val), lexeme, line, col)
        } else {
            let val = normalized.parse::<i64>().unwrap_or(0);
            Token::new(TokenType::IntLiteral(val), lexeme, line, col)
        }
    }
}

pub fn is_bangla_digit(c: char) -> bool {
    (c as u32) >= 0x09E6 && (c as u32) <= 0x09EF
}

fn map_bangla_digit(c: char) -> Option<char> {
    let code = c as u32;
    if (0x09E6..=0x09EF).contains(&code) {
        // Offset from 0x09E6 to '0' (0x30)
        let digit = (code - 0x09E6) + 0x30;
        std::char::from_u32(digit)
    } else {
        None
    }
}

pub fn is_bangla_char(c: char) -> bool {
    // Bangla Unicode block is U+0980 to U+09FF
    (c as u32) >= 0x0980 && (c as u32) <= 0x09FF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("ফাংশন ধরি যদি নাহলে যতক্ষণ ফেরত শুরু");
        assert_eq!(lexer.next_token().token_type, TokenType::Phangshon);
        assert_eq!(lexer.next_token().token_type, TokenType::Dhori);
        assert_eq!(lexer.next_token().token_type, TokenType::Jodi);
        assert_eq!(lexer.next_token().token_type, TokenType::Nahle);
        assert_eq!(lexer.next_token().token_type, TokenType::Jotokhkhon);
        assert_eq!(lexer.next_token().token_type, TokenType::Ferat);
        assert_eq!(lexer.next_token().token_type, TokenType::Shuru);
    }

    #[test]
    fn test_types() {
        let mut lexer = Lexer::new("সংখ্যা দশমিক লেখা সত্যমিথ্যা");
        assert_eq!(lexer.next_token().token_type, TokenType::Songkhya);
        assert_eq!(lexer.next_token().token_type, TokenType::Doshomik);
        assert_eq!(lexer.next_token().token_type, TokenType::Lekha);
        assert_eq!(lexer.next_token().token_type, TokenType::SottoMittha);
    }

    #[test]
    fn test_bangla_digits() {
        let mut lexer = Lexer::new("১০ ২০.৫ ০");

        let t1 = lexer.next_token();
        match t1.token_type {
            TokenType::IntLiteral(n) => assert_eq!(n, 10),
            _ => panic!("Expected IntLiteral(10), got {:?}", t1),
        }

        let t2 = lexer.next_token();
        match t2.token_type {
            TokenType::FloatLiteral(n) => assert_eq!(n, 20.5),
            _ => panic!("Expected FloatLiteral(20.5), got {:?}", t2),
        }

        let t3 = lexer.next_token();
        match t3.token_type {
            TokenType::IntLiteral(n) => assert_eq!(n, 0),
            _ => panic!("Expected IntLiteral(0), got {:?}", t3),
        }
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("নাম বয়স_১ ফল");
        assert_eq!(
            lexer.next_token().token_type,
            TokenType::Identifier("নাম".to_string())
        );
        assert_eq!(
            lexer.next_token().token_type,
            TokenType::Identifier("বয়স_১".to_string())
        );
        assert_eq!(
            lexer.next_token().token_type,
            TokenType::Identifier("ফল".to_string())
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != <= >= -> =");
        assert_eq!(lexer.next_token().token_type, TokenType::Plus);
        assert_eq!(lexer.next_token().token_type, TokenType::Minus);
        assert_eq!(lexer.next_token().token_type, TokenType::Star);
        assert_eq!(lexer.next_token().token_type, TokenType::Slash);
        assert_eq!(lexer.next_token().token_type, TokenType::EqualEqual);
        assert_eq!(lexer.next_token().token_type, TokenType::NotEqual);
        assert_eq!(lexer.next_token().token_type, TokenType::LessEqual);
        assert_eq!(lexer.next_token().token_type, TokenType::GreaterEqual);
        assert_eq!(lexer.next_token().token_type, TokenType::Arrow);
        assert_eq!(lexer.next_token().token_type, TokenType::Equal);
    }

    #[test]
    fn test_array_tokens() {
        let input = "তালিকা [১, ২, ৩]";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().token_type, TokenType::Talika);
        assert_eq!(lexer.next_token().token_type, TokenType::LBracket);
        assert_eq!(lexer.next_token().token_type, TokenType::IntLiteral(1));
        assert_eq!(lexer.next_token().token_type, TokenType::Comma);
        assert_eq!(lexer.next_token().token_type, TokenType::IntLiteral(2));
        assert_eq!(lexer.next_token().token_type, TokenType::Comma);
        assert_eq!(lexer.next_token().token_type, TokenType::IntLiteral(3));
        assert_eq!(lexer.next_token().token_type, TokenType::RBracket);
    }
}

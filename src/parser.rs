use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token {
    Text(String),
    Chinese(String),
    English(String),
    Number(String),
    InlineMath(String),
    InlineCode(String),
    BlockMath(String),
    CodeBlock{
        language: String,
        content: String
    },
    NewLine,
}

pub struct Parser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Parser { chars, current }
    }

    fn next_char(&mut self) -> Option<char> {
        let current = self.current;
        self.current = self.chars.next();
        current
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    pub fn parse(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek() {
            match c {
                '$' => {
                    self.next_char(); // consume $
                    if self.peek() == Some('$') {
                        self.next_char(); // consume second $
                        tokens.push(self.parse_block_math());
                    } else {
                        tokens.push(self.parse_inline_math());
                    }
                }
                '`' => {
                    self.next_char(); // consume `
                    if self.peek() == Some('`') {
                        self.next_char(); // consume second `
                        if self.peek() == Some('`') {
                            self.next_char(); // consume third `
                            tokens.push(self.parse_code_block());
                        }
                    } else {
                        tokens.push(self.parse_inline_code());
                    }
                }
                '\n' => {
                    tokens.push(Token::NewLine);
                    self.next_char();
                }
                c if c.is_ascii_alphabetic() => {
                    tokens.push(self.parse_english());
                }
                c if c.is_ascii_digit() => {
                    tokens.push(self.parse_number());
                }
                c if is_chinese(c) => {
                    tokens.push(self.parse_chinese());
                }
                _ => {
                    tokens.push(Token::Text(self.next_char().unwrap().to_string()));
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            println!("\n=== Token 解析结果 ===");
            for (i, token) in tokens.iter().enumerate() {
                println!("{:3}. {:?}", i, token);
            }
            println!("===================\n");
        }
        tokens
    }

    fn parse_inline_math(&mut self) -> Token {
        let mut content = String::new();
        while let Some(c) = self.next_char() {
            if c == '$' {
                break;
            }
            content.push(c);
        }
        Token::InlineMath(content)
    }

    fn parse_block_math(&mut self) -> Token {
        let mut content = String::new();
        while let Some(c) = self.next_char() {
            if c == '$' && self.peek() == Some('$') {
                self.next_char(); // consume second $
                break;
            }
            content.push(c);
        }
        Token::BlockMath(content)
    }

    fn parse_inline_code(&mut self) -> Token {
        let mut content = String::new();
        while let Some(c) = self.next_char() {
            if c == '`' {
                break;
            }
            content.push(c);
        }
        Token::InlineCode(content)
    }

    fn parse_code_block(&mut self) -> Token {
        let mut language = String::new();
        let mut content = String::new();

        // Parse language
        while let Some(c) = self.next_char() {
            if c == '\n' {
                break;
            }
            language.push(c);
        }

        // Parse content
        while let Some(c) = self.next_char() {
            if c == '`' && self.peek() == Some('`') {
                self.next_char(); // consume second `
                if self.peek() == Some('`') {
                    self.next_char(); // consume third `
                    break;
                }
            }
            content.push(c);
        }

        Token::CodeBlock {
            language: language.trim().to_string(),
            content
        }
    }

    fn parse_english(&mut self) -> Token {
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            content.push(self.next_char().unwrap());
        }
        Token::English(content)
    }

    fn parse_number(&mut self) -> Token {
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() && c != '.' {
                break;
            }
            content.push(self.next_char().unwrap());
        }
        Token::Number(content)
    }

    fn parse_chinese(&mut self) -> Token {
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if !is_chinese(c) {
                break;
            }
            content.push(self.next_char().unwrap());
        }
        Token::Chinese(content)
    }
}

fn is_chinese(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9fff}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let input = "Hello 你好 123 $x+y$ `code` $$math$$ ```rust\nfn main(){}\n```";
        let mut parser = Parser::new(input);
        let tokens = parser.parse();
        println!("{:?}", tokens);
    }
}
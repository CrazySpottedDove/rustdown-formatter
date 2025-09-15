use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    Chinese(&'a str),
    English(&'a str),
    Number(&'a str),
    InlineMath(&'a str),
    InlineCode(&'a str),
    BlockMath(&'a str),
    CodeBlock { language: &'a str, content: &'a str },
    NewLine,
}

pub struct Parser<'a> {
    input: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    byte_pos: usize, // 字节位置
    char_pos: usize, // 字符位置
    inside_quote_block: bool,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Parser {
            input,
            chars,
            current,
            byte_pos: 0,
            char_pos: 0,
            inside_quote_block: false,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let current = self.current;
        if let Some(c) = current {
            self.byte_pos += c.len_utf8();
            self.char_pos += 1;
        }
        self.current = self.chars.next();
        current
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.current
    }

    fn is_quote_start(&self) -> bool {
        // 获取当前位置到行首的切片
        let line_start = self.input[..self.byte_pos].rfind('\n').map_or(0, |n| n + 1);
        let current_line = &self.input[line_start..self.byte_pos];

        // 检查是否只包含空白字符，且当前字符是 >
        current_line.chars().all(char::is_whitespace) && self.current == Some('>')
    }

    #[inline]
    fn take_slice(&self, start_byte: usize, end_byte: usize) -> &'a str {
        &self.input[start_byte..end_byte]
    }

    #[inline]
    fn is_math_block_start(&self) -> bool {
        self.current == Some('$') && self.peek_next() == Some('$')
    }

    pub fn parse(&mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::with_capacity(self.input.len() / 4);
        let mut start_byte = 0;

        while let Some(c) = self.peek() {
            if self.is_quote_start() {
                self.inside_quote_block = true;
            } else if c == '\n' {
                self.inside_quote_block = false;
            }
            if self.inside_quote_block {
                match c {
                    '$' => {
                        if self.is_math_block_start() {
                            self.next_char();
                            self.next_char();
                        } else {
                            // 如果有待处理的文本
                            if self.byte_pos > start_byte {
                                tokens
                                    .push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                            }
                            self.next_char(); // 消费 $

                            let token = self.parse_inline_math();
                            tokens.push(token);
                            start_byte = self.byte_pos;
                        }
                    }
                    '`' if !self.is_code_block_start() => {
                        if self.is_code_block_start() {
                            self.next_char();
                            self.next_char();
                            self.next_char();
                        } else {
                            // 如果有待处理的文本
                            if self.byte_pos > start_byte {
                                tokens
                                    .push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                            }
                            self.next_char(); // 消费 `

                            let token = self.parse_inline_code();
                            tokens.push(token);
                            start_byte = self.byte_pos;
                        }
                    }
                    '\n' => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        tokens.push(Token::NewLine);
                        self.next_char();
                        start_byte = self.byte_pos;
                    }
                    c if c.is_ascii_alphabetic() => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = self.parse_english();
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    c if c.is_ascii_digit() => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = self.parse_number();
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    c if is_chinese(c) => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = self.parse_chinese();
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    _ => {
                        self.next_char();
                    }
                }
            } else {
                match c {
                    '$' => {
                        // 如果有待处理的文本
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = if self.peek_next() == Some('$') {
                            self.next_char(); // 消费第一个 $
                            self.next_char(); // 消费第二个 $
                            self.parse_block_math()
                        } else {
                            self.next_char(); // 消费 $
                            self.parse_inline_math()
                        };
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    '`' => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = if self.is_code_block_start() {
                            self.parse_code_block()
                        } else {
                            self.next_char();
                            self.parse_inline_code()
                        };
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    '\n' => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        tokens.push(Token::NewLine);
                        self.next_char();
                        start_byte = self.byte_pos;
                    }
                    c if c.is_ascii_alphabetic() => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = self.parse_english();
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    c if c.is_ascii_digit() => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = self.parse_number();
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    c if is_chinese(c) => {
                        if self.byte_pos > start_byte {
                            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
                        }
                        let token = self.parse_chinese();
                        tokens.push(token);
                        start_byte = self.byte_pos;
                    }
                    _ => {
                        self.next_char();
                    }
                }
            }
        }

        // 处理剩余的文本
        if self.byte_pos > start_byte {
            tokens.push(Token::Text(self.take_slice(start_byte, self.byte_pos)));
        }

        // 如果是debug模式，将tokens解析情况写入一个测试文件中
        #[cfg(debug_assertions)]
        {
            use std::fs::File;
            use std::io::Write;
            let mut file = File::create("debug_tokens.txt").unwrap();
            for token in &tokens {
                writeln!(file, "{:?}", token).unwrap();
            }
        }
        tokens
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next()
    }

    fn is_code_block_start(&self) -> bool {
        self.current == Some('`')
            && self.peek_next() == Some('`')
            && self.chars.clone().nth(1) == Some('`')
    }

    fn parse_inline_math(&mut self) -> Token<'a> {
        let start = self.byte_pos;
        while let Some(c) = self.next_char() {
            if c == '$' {
                return Token::InlineMath(self.take_slice(start, self.byte_pos - 1));
            }
        }
        Token::InlineMath(self.take_slice(start, self.byte_pos))
    }

    fn parse_block_math(&mut self) -> Token<'a> {
        let start = self.byte_pos;
        while let Some(c) = self.next_char() {
            if c == '$' && self.peek() == Some('$') {
                let end = self.byte_pos - 1;
                self.next_char(); // 消费第二个 $
                return Token::BlockMath(self.take_slice(start, end));
            }
        }
        Token::BlockMath(self.take_slice(start, self.byte_pos))
    }

    fn parse_inline_code(&mut self) -> Token<'a> {
        let start = self.byte_pos;
        while let Some(c) = self.next_char() {
            if c == '`' {
                return Token::InlineCode(self.take_slice(start, self.byte_pos - 1));
            }
        }
        Token::InlineCode(self.take_slice(start, self.byte_pos))
    }

    fn parse_code_block(&mut self) -> Token<'a> {
        self.next_char(); // 消费第一个 `
        self.next_char(); // 消费第二个 `
        self.next_char(); // 消费第三个 `

        let lang_start = self.byte_pos;
        let mut lang_end = lang_start;

        // 解析语言标识符
        while let Some(c) = self.next_char() {
            if c == '\n' {
                lang_end = self.byte_pos - 1;
                break;
            }
        }

        let content_start = self.byte_pos;
        while let Some(c) = self.next_char() {
            if c == '`' && self.is_code_block_end() {
                let content_end = self.byte_pos - 1;
                self.next_char(); // 消费第二个 `
                self.next_char(); // 消费第三个 `
                return Token::CodeBlock {
                    language: self.take_slice(lang_start, lang_end).trim(),
                    content: self.take_slice(content_start, content_end),
                };
            }
        }

        Token::CodeBlock {
            language: self.take_slice(lang_start, lang_end).trim(),
            content: self.take_slice(content_start, self.byte_pos),
        }
    }

    fn is_code_block_end(&self) -> bool {
        self.peek_next() == Some('`') && self.chars.clone().nth(0) == Some('`')
    }

    fn parse_english(&mut self) -> Token<'a> {
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            self.next_char();
        }
        Token::English(self.take_slice(start, self.byte_pos))
    }

    fn parse_number(&mut self) -> Token<'a> {
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() && c != '.' {
                break;
            }
            self.next_char();
        }
        Token::Number(self.take_slice(start, self.byte_pos))
    }

    fn parse_chinese(&mut self) -> Token<'a> {
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if !is_chinese(c) {
                break;
            }
            self.next_char();
        }
        Token::Chinese(self.take_slice(start, self.byte_pos))
    }
}

#[inline]
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

    #[test]
    fn test_utf8_handling() {
        let input = "你好world";
        let mut parser = Parser::new(input);
        let tokens = parser.parse();
        assert!(matches!(tokens[0], Token::Chinese(s) if s == "你好"));
        assert!(matches!(tokens[1], Token::English(s) if s == "world"));
    }

    #[test]
    fn test_mixed_text() {
        let input = "Hello世界123";
        let mut parser = Parser::new(input);
        let tokens = parser.parse();
        assert!(matches!(tokens[0], Token::English(s) if s == "Hello"));
        assert!(matches!(tokens[1], Token::Chinese(s) if s == "世界"));
        assert!(matches!(tokens[2], Token::Number(s) if s == "123"));
    }
}

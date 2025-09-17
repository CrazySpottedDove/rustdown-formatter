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
    Title(Vec<Token<'a>>, usize), // (text, level)
    FakeCodeBlock,                // 用于占位，表示这是一个代码块，实际内容在 code_block_tokens 中
}

pub struct CodeBlock<'a>{
    pub language: &'a str,
    pub content: &'a str,
}

pub struct Parser<'a> {
    input: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    byte_pos: usize, // 字节位置
    char_pos: usize, // 字符位置
    inside_quote_block: bool,
    text_start_byte: usize, // 当前文本块的起始字节位置,
    tokens: Vec<Token<'a>>,
    code_blocks: Vec<CodeBlock<'a>>, // 用于存储代码块 tokens，用于之后并行处理
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
            text_start_byte: 0,
            tokens: Vec::with_capacity(input.len() / 4),
            code_blocks: Vec::new(),
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token<'a>> {
        &self.tokens
    }

    pub fn get_code_blocks(&self) -> &Vec<CodeBlock<'a>> {
        &self.code_blocks
    }

    fn jump_next_char(&mut self) {
        let current = self.current;
        if let Some(c) = current {
            self.byte_pos += c.len_utf8();
            self.char_pos += 1;
        }
        self.current = self.chars.next();
    }

    fn get_next_char(&mut self) -> Option<char> {
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

    #[inline]
    fn take_slice(&self, start_byte: usize, end_byte: usize) -> &'a str {
        &self.input[start_byte..end_byte]
    }

    fn judge_quote_start(&mut self) {
        for token in self.tokens.iter().rev() {
            match token {
                Token::NewLine => {
                    self.inside_quote_block = true;
                    break;
                }
                Token::Text(s) if s.chars().all(char::is_whitespace) => continue,
                _ => {
                    break;
                }
            }
        }
    }

    fn parse_title(&mut self) {
        self.flush_text();
        let mut level = 0;
        // 扫描全部的 #
        while let Some(c) = self.peek() {
            if c == '#' {
                level += 1;
                self.jump_next_char();
            } else {
                break;
            }
        }
        if let Some(c) = self.current {
            if !c.is_whitespace() {
                // 不是标题
                self.text_start_byte = self.byte_pos - level; // 回退到 # 位置
                return;
            }
        }
        for token in self.tokens.iter().rev() {
            match token {
                Token::NewLine => {
                    // 是标题
                    break;
                }
                Token::Text(s) if s.chars().all(char::is_whitespace) => continue,
                _ => {
                    // 不是标题
                    self.text_start_byte = self.byte_pos - level; // 回退到 # 位置
                    return;
                }
            }
        }
        // 跳过标题标记后的空格
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.jump_next_char();
            } else {
                break;
            }
        }
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            self.jump_next_char();
        }
        let title_text = self.take_slice(start, self.byte_pos).trim();
        let mut title_parser = Parser::new(title_text);
        title_parser.parse();
        let title_tokens = title_parser.tokens;
        self.tokens.push(Token::Title(title_tokens, level));
        self.text_start_byte = self.byte_pos;
    }

    pub fn parse(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                '>' => {
                    self.judge_quote_start();
                    self.jump_next_char();
                }
                '$' => self.parse_math(),
                '`' => self.parse_code(),
                '\r' => {
                    self.flush_text();
                    self.jump_next_char();
                    self.text_start_byte = self.byte_pos;
                }
                '\n' => {
                    self.flush_text();
                    self.jump_next_char();
                    self.tokens.push(Token::NewLine);
                    self.text_start_byte = self.byte_pos;
                    self.inside_quote_block = false;
                }
                '#' => {
                    self.parse_title();
                }
                c if c.is_ascii_alphabetic() => self.parse_english(),
                c if c.is_ascii_digit() => self.parse_number(),
                c if is_chinese(c) => self.parse_chinese(),
                _ => self.jump_next_char(),
            }
        }

        // 处理剩余的文本
        self.flush_text();

        // 如果是debug模式，将tokens解析情况写入一个测试文件中
        #[cfg(debug_assertions)]
        {
            use std::fs::File;
            use std::io::Write;
            let mut file = File::create("debug_tokens.txt").unwrap();
            for token in &self.tokens {
                writeln!(file, "{:?}", token).unwrap();
            }
        }
    }

    fn flush_text(&mut self) {
        if self.byte_pos > self.text_start_byte {
            self.tokens.push(Token::Text(
                self.take_slice(self.text_start_byte, self.byte_pos),
            ));
        }
    }

    fn parse_math(&mut self) {
        self.flush_text();
        self.jump_next_char();
        if self.current == Some('$') {
            self.jump_next_char();
            if !self.inside_quote_block {
                self.parse_block_math();
            } else {
                self.text_start_byte = self.byte_pos - 2; // 跳过 $$
                return;
            }
        } else {
            self.parse_inline_math();
        }
        self.text_start_byte = self.byte_pos;
    }

    fn parse_code(&mut self) {
        self.flush_text();
        self.jump_next_char();
        if self.current == Some('`') && self.peek_next() == Some('`') {
            self.jump_next_char();
            self.jump_next_char();
            if !self.inside_quote_block {
                self.parse_code_block();
            } else {
                self.text_start_byte = self.byte_pos - 3; // 跳过 ```
                return;
            }
        } else {
            self.parse_inline_code();
        }
        self.text_start_byte = self.byte_pos;
    }

    fn parse_english(&mut self) {
        self.flush_text();
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            self.jump_next_char();
        }
        self.tokens
            .push(Token::English(self.take_slice(start, self.byte_pos)));
        self.text_start_byte = self.byte_pos;
    }
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next()
    }

    fn parse_inline_math(&mut self) {
        let start = self.byte_pos;
        while let Some(c) = self.get_next_char() {
            if c == '$' {
                self.tokens
                    .push(Token::InlineMath(self.take_slice(start, self.byte_pos - 1)));
                return;
            }
        }
        self.tokens.push(Token::InlineMath(
            self.take_slice(start, self.byte_pos).trim(),
        ));
    }

    fn parse_block_math(&mut self) {
        let start = self.byte_pos;
        while let Some(c) = self.get_next_char() {
            if c == '$' && self.peek() == Some('$') {
                let end = self.byte_pos - 1;
                self.jump_next_char(); // 消费第二个 $
                self.tokens
                    .push(Token::BlockMath(self.take_slice(start, end)));
                return;
            }
        }
        self.tokens
            .push(Token::BlockMath(self.take_slice(start, self.byte_pos)));
    }

    fn parse_inline_code(&mut self) {
        let start = self.byte_pos;
        while let Some(c) = self.get_next_char() {
            if c == '`' {
                self.tokens.push(Token::InlineCode(
                    self.take_slice(start, self.byte_pos - 1).trim(),
                ));
                return;
            }
        }
        self.tokens
            .push(Token::InlineCode(self.take_slice(start, self.byte_pos)));
    }

    fn parse_code_block(&mut self) {
        let lang_start = self.byte_pos;
        let mut lang_end = lang_start;

        // 解析语言标识符
        while let Some(c) = self.get_next_char() {
            if c == '\n' {
                lang_end = self.byte_pos - 1;
                break;
            }
        }
        let lang = normalize_language(self.take_slice(lang_start, lang_end).trim());
        let content_start = self.byte_pos;
        while let Some(c) = self.get_next_char() {
            if c == '`' && self.peek_next() == Some('`') && self.chars.clone().nth(0) == Some('`') {
                let content_end = self.byte_pos - 1;
                self.jump_next_char(); // 消费第二个 `
                self.jump_next_char(); // 消费第三个 `
                self.code_blocks.push(CodeBlock {
                    language: lang,
                    content: self.take_slice(content_start, content_end),
                });
                self.tokens.push(Token::FakeCodeBlock); // 占位符
                return;
            }
        }
        self.code_blocks.push(CodeBlock {
            language: lang,
            content: self.take_slice(content_start, self.byte_pos),
        });

        self.tokens.push(Token::FakeCodeBlock); // 占位符
    }

    fn parse_number(&mut self) {
        self.flush_text();
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() && c != '.' {
                break;
            }
            self.jump_next_char();
        }
        self.tokens
            .push(Token::Number(self.take_slice(start, self.byte_pos)));
        self.text_start_byte = self.byte_pos;
    }

    fn parse_chinese(&mut self) {
        self.flush_text();
        let start = self.byte_pos;
        while let Some(c) = self.peek() {
            if !is_chinese(c) {
                break;
            }
            self.jump_next_char();
        }
        self.tokens
            .push(Token::Chinese(self.take_slice(start, self.byte_pos)));
        self.text_start_byte = self.byte_pos;
    }
}

#[inline]
fn is_chinese(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9fff}')
}

fn normalize_language(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        "javascript" => "js",
        "typescript" => "ts",
        "python" => "py",
        "c++" | "cxx" => "cpp",
        "golang" => "go",
        "rb" => "ruby",
        "yaml" => "yml",
        "latex" => "tex",
        "markdown" => "md",
        "sqlite" => "sql",
        "shell" => "sh",
        "bash" => "sh",
        "zsh" => "sh",
        "kt" => "kotlin",
        "sass" => "scss",
        _ => lang,
    }
}

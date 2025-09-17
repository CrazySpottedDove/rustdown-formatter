use crate::config::Config;
use crate::parser::{CodeBlock, Token};
use crate::pipeline::format_string;

use anyhow::{Result, anyhow};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::Path;
use std::process::Command;
use tex_fmt::args::Args;
use tex_fmt::format::format_file;
use tex_fmt::logging::Log;

pub struct Formatter<'a> {
    config: &'a Config,
    latex_args: Args,
    latex_logs: Vec<Log>,
    output: String,
}

fn get_formatter_command(
    formatter: &str,
    language: &str,
) -> Option<(&'static str, Vec<&'static str>)> {
    match formatter {
        "prettier" => match language {
            "js" => Some(("prettier", vec!["--std", "--parser", "babel"])),
            "ts" => Some(("prettier", vec!["--std", "--parser", "typescript"])),
            "css" => Some(("prettier", vec!["--std", "--parser", "css"])),
            "scss" => Some(("prettier", vec!["--std", "--parser", "scss"])),
            "less" => Some(("prettier", vec!["--std", "--parser", "less"])),
            "html" => Some(("prettier", vec!["--std", "--parser", "html"])),
            "json" => Some(("prettier", vec!["--std", "--parser", "json"])),
            "yml" => Some(("prettier", vec!["--std", "--parser", "yaml"])),
            "graphql" | "gql" => Some(("prettier", vec!["--std", "--parser", "graphql"])),
            "vue" => Some(("prettier", vec!["--std", "--parser", "vue"])),
            "angular" => Some(("prettier", vec!["--std", "--parser", "angular"])),
            _ => None,
        },
        "rustfmt" => Some(("rustfmt", vec!["--edition", "2021"])),
        "gofmt" => Some(("gofmt", vec![])),
        "black" => Some((
            "black",
            vec!["-"], // 使用 - 表示从 stdin 读取
        )),
        "clang-format" => {
            let style_arg = match language {
                "c" | "cpp" | "c++" | "java" | "js" | "javascript" => "--style=Google",
                _ => "--style=LLVM",
            };
            Some(("clang-format", vec![style_arg]))
        }
        "shfmt" => Some((
            "shfmt",
            vec!["-i", "2"], // 2 spaces indentation
        )),
        "sqlfmt" => Some(("sqlfmt", vec!["-"])),
        "terraform" => Some(("terraform", vec!["fmt", "-"])),
        "stylua" => Some((
            "stylua",
            vec!["-"], // 从 stdin 读取
        )),
        "dartfmt" => Some((
            "dart",
            vec!["format"], // dart format 默认从 stdin 读取
        )),
        "php-cs-fixer" => Some((
            "php-cs-fixer",
            vec!["fix", "--using-cache=no", "-"], // 使用 - 表示 stdin
        )),
        "isort" => Some((
            "isort",
            vec!["-"], // 从 stdin 读取
        )),
        "autopep8" => Some((
            "autopep8",
            vec!["-"], // 从 stdin 读取
        )),
        "yapf" => Some((
            "yapf",
            vec!["-"], // 从 stdin 读取
        )),
        "scalafmt" => Some((
            "scalafmt",
            vec!["--stdin"], // 显式指定 stdin
        )),
        "ktfmt" => Some((
            "ktfmt",
            vec!["--stdin"], // 从 stdin 读取
        )),
        _ => None,
    }
}

impl<'a> Formatter<'a> {
    pub fn new(config: &'a Config) -> Self {
        Formatter {
            config,
            latex_args: Args::default(),
            latex_logs: Vec::new(),
            output: String::new(),
        }
    }
    pub fn get_output(self) -> String {
        self.output
    }

    fn format_chinese(&mut self, text: &str, prev_token: &Option<&Token>) {
        if (self.config.space_between_zh_and_en && matches!(prev_token, Some(Token::English(_))))
            || (self.config.space_between_zh_and_num
                && matches!(prev_token, Some(Token::Number(_))))
            || matches!(prev_token, Some(Token::InlineMath(_)))
            || matches!(prev_token, Some(Token::InlineCode(_)))
        {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    fn format_english(&mut self, text: &str, prev_token: &Option<&Token>) {
        if (self.config.space_between_zh_and_en && matches!(prev_token, Some(Token::Chinese(_))))
            || matches!(prev_token, Some(Token::InlineMath(_)))
            || matches!(prev_token, Some(Token::InlineCode(_)))
        {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    fn format_number(&mut self, text: &str, prev_token: &Option<&Token>) {
        if (self.config.space_between_zh_and_num && matches!(prev_token, Some(Token::Chinese(_))))
            || matches!(prev_token, Some(Token::InlineMath(_)))
            || matches!(prev_token, Some(Token::InlineCode(_)))
        {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    fn format_inline_math(&mut self, text: &str, prev_token: &Option<&Token>) {
        if let Some(prev) = prev_token {
            match prev {
                Token::Chinese(_) | Token::English(_) | Token::Number(_) => {
                    self.output.push(' ');
                }
                _ => {}
            }
        }
        self.output.push('$');
        self.output.push_str(text);
        self.output.push('$');
    }

    fn format_block_math(&mut self, text: &str) {
        self.ensure_empty_line();
        let output = &mut self.output;
        output.push_str("$$\n");
        output.push_str(&format_file(
            text,
            Path::new("tex-fmt.log"),
            &self.latex_args,
            &mut self.latex_logs,
        ));
        output.push_str("\n$$");
        self.ensure_empty_line();
    }

    fn format_block_code_par(config: &Config, language: &str, content: &str) -> String {
        let output = &mut String::new();
        output.push_str("```");
        output.push_str(language);
        if !content.starts_with('\n') {
            output.push('\n');
        }
        if config.format_code_block {
            if language == "tex" {
                output.push_str(&format_file(
                    content,
                    Path::new("tex-fmt.log"),
                    &Args::default(),
                    &mut Vec::new(),
                ));
            } else if language == "md" {
                output.push_str(&format_string(content, config));
            } else {
                if let Some(formatter) = config.code_formatters.get(language) {
                    if let Some((cmd, args)) = get_formatter_command(formatter, language) {
                        match format_with_command(&cmd, &args, content) {
                            Ok(formatted) => {
                                output.push_str(&formatted);
                            }
                            Err(e) => {
                                eprintln!("Failed to format code block: {}", e);
                                output.push_str(content);
                            }
                        }
                    } else {
                        output.push_str(content);
                    }
                } else {
                    output.push_str(content);
                }
            }
        } else {
            output.push_str(content);
        }
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("```");
        std::mem::take(output)
    }

    fn format_block_code(&mut self, language: &str, content: &str) {
        self.ensure_empty_line();
        let output = &mut self.output;
        output.push_str("```");
        output.push_str(language);
        if !content.starts_with('\n') {
            output.push('\n');
        }
        if self.config.format_code_block {
            if language == "tex" {
                output.push_str(&format_file(
                    content,
                    Path::new("tex-fmt.log"),
                    &self.latex_args,
                    &mut self.latex_logs,
                ));
            } else if language == "md" {
                output.push_str(&format_string(content, self.config));
            } else {
                if let Some(formatter) = self.config.code_formatters.get(language) {
                    if let Some((cmd, args)) = get_formatter_command(formatter, language) {
                        match format_with_command(&cmd, &args, content) {
                            Ok(formatted) => {
                                output.push_str(&formatted);
                            }
                            Err(e) => {
                                eprintln!("Failed to format code block: {}", e);
                                output.push_str(content);
                            }
                        }
                    } else {
                        output.push_str(content);
                    }
                } else {
                    output.push_str(content);
                }
            }
        } else {
            output.push_str(content);
        }
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("```");
        self.ensure_empty_line();
    }

    fn format_inline_code(&mut self, text: &str, prev_token: &Option<&Token>) {
        if let Some(prev) = prev_token {
            match prev {
                Token::Chinese(_) | Token::English(_) | Token::Number(_) => {
                    self.output.push(' ');
                }
                _ => {}
            }
        }
        self.output.push('`');
        self.output.push_str(text);
        self.output.push('`');
    }

    fn format_title(&mut self, title_tokens: &Vec<Token<'a>>, level: &usize) {
        let mut title_formatter = Formatter::new(self.config);
        title_formatter.format(title_tokens, &vec![]);
        let title_content = title_formatter.get_output();
        let hashes = "#".repeat(*level);
        self.ensure_empty_line();
        let output = &mut self.output;
        output.push_str(&hashes);
        output.push(' ');
        output.push_str(&title_content);
        self.ensure_empty_line();
    }

    pub fn format(&mut self, tokens: &Vec<Token<'a>>, code_blocks: &Vec<CodeBlock>) {
        self.output.reserve(tokens.len() * 3);
        let mut prev_token: Option<&Token> = None;
        let code_block_formatted_strings = code_blocks
            .par_iter()
            .map(|code_block| {
                Formatter::format_block_code_par(
                    self.config,
                    code_block.language,
                    code_block.content,
                )
            })
            .collect::<Vec<String>>();
        let mut code_block_id = 0;

        for token in tokens.iter() {
            match token {
                Token::Chinese(text) => self.format_chinese(text, &prev_token),
                Token::English(text) => self.format_english(text, &prev_token),
                Token::Number(text) => self.format_number(text, &prev_token),
                Token::InlineMath(text) => self.format_inline_math(text, &prev_token),
                Token::BlockMath(text) => self.format_block_math(text),
                Token::FakeCodeBlock => {
                    self.ensure_empty_line();
                    let formatted = &code_block_formatted_strings[code_block_id];
                    self.output.push_str(formatted);
                    code_block_id += 1;
                    self.ensure_empty_line();
                }
                Token::CodeBlock { language, content } => self.format_block_code(language, content),
                Token::InlineCode(text) => self.format_inline_code(text, &prev_token),
                Token::NewLine => {
                    if !self.output.ends_with("\n\n") {
                        self.output.push('\n');
                    }
                }
                Token::Text(text) => {
                    self.output.push_str(text);
                }
                Token::Title(title_tokens, level) => self.format_title(title_tokens, level),
            }
            prev_token = Some(token);
        }
    }

    fn ensure_empty_line(&mut self) {
        let output = &mut self.output;
        if output.is_empty() {
            return;
        }
        while output.ends_with('\n') {
            output.pop();
        }
        output.push_str("\n\n");
    }
}

fn format_with_command(cmd: &str, args: &[&'static str], content: &str) -> Result<String> {
    #[cfg(debug_assertions)]
    {
        eprintln!("[DEBUG] Try to run external formatter: {} {:?}", cmd, args);
    }

    let mut child = match Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            if let Some(2) = e.raw_os_error() {
                // os error 2: No such file or directory
                return Err(anyhow!("未找到格式化工具 `{}`。", cmd));
            } else {
                return Err(anyhow!("无法启动格式化工具 `{}`: {}", cmd, e));
            }
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(content.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("{error_message}"))
    }
}

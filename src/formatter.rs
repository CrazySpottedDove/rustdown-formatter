use crate::config::Config;
use crate::parser::{Parser, Token};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::process::Command;
use tex_fmt::args::Args;
use tex_fmt::format::format_file;
use tex_fmt::logging::Log;
pub struct Formatter {
    config: Config,
    latex_args: Args,
    latex_logs: Vec<Log>,
}

impl Formatter {
    pub fn new(config: Config) -> Self {
        Formatter {
            config,
            latex_args: Args::default(),
            latex_logs: Vec::new(),
        }
    }
    #[inline]
    fn format_math(&mut self, text: &str) -> String {
        #[cfg(debug_assertions)]
        {
            eprintln!("[DEBUG] format_math: input = {:?}", text);
        }
        let formatted = format_file(
            text,
            Path::new("tex-fmt.log"),
            &self.latex_args,
            &mut self.latex_logs,
        );
        #[cfg(debug_assertions)]
        {
            eprintln!("[DEBUG] format_math: output = {:?}", formatted);
        }
        formatted
    }

    pub fn format(&mut self, input: &str) -> String {
        let mut parser = Parser::new(input);
        let tokens = parser.parse();
        let mut result = String::with_capacity(input.len() * 6 / 5);
        let mut prev_token: Option<&Token> = None;

        for token in tokens.iter() {
            match token {
                Token::Chinese(text) => {
                    // 处理中文与英文之间的空格
                    if self.config.space_between_zh_and_en {
                        if let Some(Token::English(_)) = prev_token {
                            result.push(' ');
                        }
                    }
                    // 处理中文与数字之间的空格
                    if self.config.space_between_zh_and_num {
                        if let Some(Token::Number(_)) = prev_token {
                            result.push(' ');
                        }
                    }

                    if let Some(Token::InlineMath(_)) = prev_token {
                        result.push(' ');
                    }

                    if let Some(Token::InlineCode(_)) = prev_token {
                        result.push(' ');
                    }

                    result.push_str(text);
                }
                Token::English(text) => {
                    // 处理英文与中文之间的空格
                    if self.config.space_between_zh_and_en {
                        if let Some(Token::Chinese(_)) = prev_token {
                            result.push(' ');
                        }
                    }
                    if let Some(Token::InlineMath(_)) = prev_token {
                        result.push(' ');
                    }

                    if let Some(Token::InlineCode(_)) = prev_token {
                        result.push(' ');
                    }
                    result.push_str(text);
                }
                Token::Number(text) => {
                    // 处理数字与中文之间的空格
                    if self.config.space_between_zh_and_num {
                        if let Some(Token::Chinese(_)) = prev_token {
                            result.push(' ');
                        }
                    }
                    if let Some(Token::InlineMath(_)) = prev_token {
                        result.push(' ');
                    }

                    if let Some(Token::InlineCode(_)) = prev_token {
                        result.push(' ');
                    }
                    result.push_str(text);
                }
                Token::InlineMath(text) => {
                    // 处理行内公式与文本之间的空格
                    if let Some(prev) = prev_token {
                        match prev {
                            Token::Chinese(_) | Token::English(_) | Token::Number(_) => {
                                result.push(' ');
                            }
                            _ => {}
                        }
                    }
                    result.push('$');
                    result.push_str(text.trim());
                    result.push('$');
                }
                Token::BlockMath(text) => {
                    #[cfg(debug_assertions)]
                    {
                        eprintln!("[DEBUG] BlockMath detected: {:?}", text);
                    }
                    ensure_empty_line(&mut result);
                    result.push_str("$$\n");

                    let formatted = self.format_math(text).trim().to_string();
                    result.push_str(&formatted);

                    result.push_str("\n$$");
                    ensure_empty_line(&mut result);
                }
                Token::CodeBlock { language, content } => {
                    ensure_empty_line(&mut result);
                    result.push_str("```");
                    result.push_str(language);
                    if !content.starts_with('\n') {
                        result.push('\n');
                    }
                    if self.config.format_code_block {
                        result.push_str(&format_code_block(&self.config, language, content));
                    } else {
                        result.push_str(content);
                    }
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push_str("```");
                    ensure_empty_line(&mut result);
                }
                Token::InlineCode(text) => {
                    if self.config.space_between_code_and_text {
                        if let Some(prev) = prev_token {
                            match prev {
                                Token::Chinese(_) | Token::English(_) | Token::Number(_) => {
                                    result.push(' ');
                                }
                                _ => {}
                            }
                        }
                    }
                    result.push('`');
                    result.push_str(text.trim());
                    result.push('`');
                }
                Token::NewLine => {
                    if !result.ends_with("\n\n") {
                        result.push('\n');
                    }
                }
                Token::Text(text) => {
                    result.push_str(text);
                }
            }
            prev_token = Some(token);
        }
        result
    }
}

fn get_formatter_command(config: &Config, language: &str) -> Option<(String, Vec<String>)> {
    if let Some(formatter) = config.code_formatters.get(language) {
        match formatter.as_str() {
            "prettier" => {
                let parser = match language {
                    "js" | "javascript" => Some("babel"),
                    "ts" | "typescript" => Some("typescript"),
                    "css" => Some("css"),
                    "html" => Some("html"),
                    "json" => Some("json"),
                    "yaml" | "yml" => Some("yaml"),
                    "markdown" | "md" => Some("markdown"),
                    _ => None,
                };

                parser.map(|p| {
                    (
                        "prettier".to_string(),
                        vec!["--parser".to_string(), p.to_string()],
                    )
                })
            }
            "rustfmt" => Some((
                "rustfmt".to_string(),
                vec!["--edition".to_string(), "2021".to_string()],
            )),
            "latexindent" => Some((
                "latexindent".to_string(),
                vec![
                    "-l".to_string(),
                    "-m".to_string(),
                    "-s".to_string(),
                    "-y=\"defaultIndent:''\"".to_string(),
                    "-y=\"removeTrailingWhitespace:1\"".to_string(),
                    "-y=\"noAdditionalIndent:1\"".to_string(),
                    "-y=\"lookForAlignDelims:0\"".to_string(),
                ],
            )),
            _ => None,
        }
    } else {
        None
    }
}

fn format_with_command(cmd: &str, args: &[String], content: &str) -> Result<String> {
    #[cfg(debug_assertions)]
    {
        eprintln!("[DEBUG] Try to run external formatter: {} {:?}", cmd, args);
    }

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(content.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        #[cfg(debug_assertions)]
        {
            eprintln!("[DEBUG] Formatter output: {}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(String::from_utf8(output.stdout)?)
    } else {
        #[cfg(debug_assertions)]
        {
            eprintln!(
                "[DEBUG] Formatter failed. Stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(anyhow!("formatter failed"))
    }
}

// ...existing code...

fn format_code_block(config: &Config, language: &str, content: &str) -> String {
    #[cfg(debug_assertions)]
    {
        eprintln!("[DEBUG] format_code_block: language = {}, content = {:?}", language, &content[..content.len().min(60)]);
    }
    let Some((cmd, args)) = get_formatter_command(config, language) else {
        #[cfg(debug_assertions)]
        {
            eprintln!("[DEBUG] No formatter configured for language: {}", language);
        }
        return content.to_string();
    };

    match format_with_command(&cmd, &args, content) {
        Ok(formatted) => formatted,
        Err(e) => {
            eprintln!("Failed to format code block: {}", e);
            content.to_string()
        }
    }
}

fn ensure_empty_line(result: &mut String) {
    while result.ends_with('\n') {
        result.pop();
    }
    result.push_str("\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let config = Config::default();
        let mut formatter = Formatter::new(config);
        let input = "Hello你好123 $x+y$ `code` $$math$$ ```rust\nfn main(){}\n```";
        let result = formatter.format(input);
        println!("Formatted:\n{}", result);
    }
}

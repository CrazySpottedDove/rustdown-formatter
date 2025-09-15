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
        format_file(
            text,
            Path::new("tex-fmt.log"),
            &self.latex_args,
            &mut self.latex_logs,
        )
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
                    // if self.config.format_math {
                    //     result.push_str(&self.format_math(&text));
                    // } else {
                    result.push_str(text.trim());
                    // }
                    result.push('$');
                }
                Token::BlockMath(text) => {
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
                    if !content.ends_with('\n') {
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

    // let Some((cmd, args)) = get_formatter_command(config, "latex") else {
    //     return text.to_string();
    // };

    // let wrapped_content = format!("\\begin{{document}}\n{}\n\\end{{document}}", text);

    // match format_with_command(&cmd, &args, &wrapped_content) {
    //     Ok(formatted) => {
    //         if let Some(start) = formatted.find("\\begin{document}\n") {
    //             if let Some(end) = formatted.find("\n\\end{document}") {
    //                 return formatted[start + 16..end].trim().to_string();
    //             }
    //         }
    //         text.to_string()
    //     }
    //     Err(e) => {
    //         eprintln!("Failed to format math: {}", e);
    //         text.to_string()
    //     }
    // }
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
                static PRETTIER_ARGS: [&str; 2] = ["--parser", ""];
                parser.map(|p| {
                    (
                        PRETTIER_ARGS[0].to_string(),
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
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow!("formatter failed"))
    }
}

fn format_code_block(config: &Config, language: &str, content: &str) -> String {
    if !config.format_code_block {
        return content.to_string();
    }

    let Some((cmd, args)) = get_formatter_command(config, language) else {
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
    let bytes = result.as_bytes();
    let mut newlines = 0;
    let mut i = bytes.len();

    while i > 0 && bytes[i - 1] == b'\n' {
        newlines += 1;
        i -= 1;
    }

    match newlines {
        0 => result.push_str("\n\n"),
        1 => result.push('\n'),
        _ => {
            result.truncate(result.len() - (newlines - 2));
        }
    }
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

use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug)]
pub struct Config{
    pub space_between_zh_and_en: bool,
    pub space_between_zh_and_num: bool,
    pub format_code_block: bool,
    pub space_between_code_and_text: bool,
    pub code_formatters: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut code_formatters = HashMap::new();

        // 添加默认的代码格式化工具
        code_formatters.insert("rust".to_string(), "rustfmt".to_string());
        code_formatters.insert("javascript".to_string(), "prettier".to_string());
        code_formatters.insert("js".to_string(), "prettier".to_string());
        code_formatters.insert("typescript".to_string(), "prettier".to_string());
        code_formatters.insert("ts".to_string(), "prettier".to_string());
        code_formatters.insert("css".to_string(), "prettier".to_string());
        code_formatters.insert("html".to_string(), "prettier".to_string());
        code_formatters.insert("json".to_string(), "prettier".to_string());
        code_formatters.insert("yaml".to_string(), "prettier".to_string());
        code_formatters.insert("yml".to_string(), "prettier".to_string());
        code_formatters.insert("markdown".to_string(), "prettier".to_string());
        code_formatters.insert("md".to_string(), "prettier".to_string());
        code_formatters.insert("latex".to_string(), "latexindent".to_string());

        Config {
            space_between_zh_and_en: true,
            space_between_zh_and_num: true,
            format_code_block: true,
            space_between_code_and_text: true,
            code_formatters,
        }
    }
}

impl Config{
    pub fn new() -> Self {
        Config::default()
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let file_content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&file_content)?;
        Ok(config)
    }
}
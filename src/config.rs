use serde::{Serialize, Deserialize};
use maplit::hashmap;
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug)]
pub struct Config{
    pub space_between_zh_and_en: bool,
    pub space_between_zh_and_num: bool,
    pub format_code_block: bool,
    pub code_formatters: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let code_formatters = hashmap! {
            "rust".to_string()      => "rustfmt".to_string(),
            "js".to_string()        => "prettier".to_string(),
            "ts".to_string()        => "prettier".to_string(),
            "css".to_string()       => "prettier".to_string(),
            "scss".to_string()      => "prettier".to_string(),
            "sass".to_string()      => "prettier".to_string(),
            "less".to_string()      => "prettier".to_string(),
            "html".to_string()      => "prettier".to_string(),
            "json".to_string()      => "prettier".to_string(),
            "yml".to_string()       => "prettier".to_string(),
            "graphql".to_string()   => "prettier".to_string(),
            "gql".to_string()       => "prettier".to_string(),
            "vue".to_string()       => "prettier".to_string(),
            "angular".to_string()   => "prettier".to_string(),
            "c".to_string()         => "clang-format".to_string(),
            "cpp".to_string()       => "clang-format".to_string(),
            "java".to_string()      => "clang-format".to_string(),
            "go".to_string()        => "gofmt".to_string(),
            "py".to_string()        => "black".to_string(),
            "sh".to_string()        => "shfmt".to_string(),
            "sql".to_string()       => "sqlfmt".to_string(),
            "tf".to_string()        => "terraform".to_string(),
            "lua".to_string()       => "stylua".to_string(),
            "dart".to_string()      => "dartfmt".to_string(),
            "php".to_string()       => "php-cs-fixer".to_string(),
            "isort".to_string()     => "isort".to_string(),
            "autopep8".to_string()  => "autopep8".to_string(),
            "yapf".to_string()      => "yapf".to_string(),
            "scala".to_string()     => "scalafmt".to_string(),
            "kotlin".to_string()    => "ktfmt".to_string(),
            // 你可以根据需要继续扩展
        };

        Config {
            space_between_zh_and_en: true,
            space_between_zh_and_num: true,
            format_code_block: true,
            code_formatters,
        }
    }
}

// impl Config{
//     pub fn new() -> Self {
//         Config::default()
//     }
//     pub fn from_file(path: &str) -> Result<Self> {
//         let file_content = std::fs::read_to_string(path)?;
//         let config: Config = serde_json::from_str(&file_content)?;
//         Ok(config)
//     }
// }
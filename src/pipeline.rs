use crate::{config::Config, parser::Parser, formatter::Formatter};

pub fn format_string(input: &str, config: &Config)->String{
    // #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();

    let mut parser = Parser::new(input);
    parser.parse();

    // #[cfg(debug_assertions)]
    let t2 = std::time::Instant::now();

    let mut formatter = Formatter::new(config);
    let tokens = parser.get_tokens();
    let code_blocks = parser.get_code_blocks();
    formatter.format(&tokens, &code_blocks);

    // #[cfg(debug_assertions)]
    let t3 = std::time::Instant::now();

    // #[cfg(debug_assertions)]
    {
        println!("Parsing time: {:?}", t2 - t1);
        println!("Formatting time: {:?}", t3 - t2);
    }
    formatter.get_output()
}
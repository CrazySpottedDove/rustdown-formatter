mod config;
mod formatter;
mod parser;

use config::Config;
use formatter::Formatter;
use std::env;
use std::fs;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("用法: rustdown-formatter <文件路径>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let path = Path::new(file_path);

    // 检查文件是否存在
    if !path.exists() {
        eprintln!("错误: 文件 '{}' 不存在", file_path);
        std::process::exit(1);
    }

    // 读取文件内容
    let t1 = std::time::Instant::now();
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let t2 = std::time::Instant::now();
    println!("读取文件耗时: {:?}", t2 - t1);

    // 格式化内容
    let config = Config::default();
    let mut formatter = Formatter::new(config);
    let formatted = formatter.format(&content);
    let t3 = std::time::Instant::now();
    println!("格式化耗时: {:?}", t3 - t2);

    // 使用缓冲写入
    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(formatted.as_bytes())?;
    writer.flush()?;  // 确保所有数据都写入文件
    let t4 = std::time::Instant::now();
    println!("写回文件耗时: {:?}", t4 - t3);
    Ok(())
}

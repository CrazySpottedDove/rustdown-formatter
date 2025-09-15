#!/bin/bash

# 编译 Rust 程序
cargo build --release

# 创建 bin 目录
mkdir -p extension/bin

# 复制可执行文件到 bin 目录
cp target/release/rustdown-formatter extension/bin/

# 进入扩展目录
cd extension

# 安装依赖
npm install

# 编译 TypeScript
npm run compile
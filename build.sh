#!/bin/bash

# 创建 dist 目录（使用绝对路径）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${SCRIPT_DIR}/dist"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# 编译两个平台的版本
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

# 创建打包目录
PACK_DIR="extension"

# 创建平台特定的二进制目录
mkdir -p "${PACK_DIR}/bin/linux"
mkdir -p "${PACK_DIR}/bin/win32"

# 复制两个平台的可执行文件
cp "target/release/rustdown-formatter" "${PACK_DIR}/bin/linux/"
chmod +x "${PACK_DIR}/bin/linux/rustdown-formatter"
cp "target/x86_64-pc-windows-gnu/release/rustdown-formatter.exe" "${PACK_DIR}/bin/win32/"

# 安装依赖并打包
cd "${PACK_DIR}"
npm install --ignore-scripts --no-audit --no-fund
npm run compile
npx vsce publish
cd ..

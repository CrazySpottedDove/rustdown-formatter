#!/bin/bash

# 创建 dist 目录（使用绝对路径）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${SCRIPT_DIR}/dist"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# 编译两个平台的版本
echo "编译 Linux 版本..."
cargo build --release
echo "编译 Windows 版本..."
cargo build --release --target x86_64-pc-windows-gnu

# 创建打包目录
PACK_DIR="extension-universal"
rm -rf "${PACK_DIR}"
mkdir -p "${PACK_DIR}"

# 复制扩展文件
cp -r extension/* "${PACK_DIR}/"

# 创建平台特定的二进制目录
mkdir -p "${PACK_DIR}/bin/linux"
mkdir -p "${PACK_DIR}/bin/win32"

# 复制两个平台的可执行文件
cp "target/release/rustdown-formatter" "${PACK_DIR}/bin/linux/"
chmod +x "${PACK_DIR}/bin/linux/rustdown-formatter"
cp "target/x86_64-pc-windows-gnu/release/rustdown-formatter.exe" "${PACK_DIR}/bin/win32/"

# 修改 package.json
jq '.name = "rustdown-formatter" | .displayName = "Rustdown Formatter" | .os = ["linux", "win32"]' "${PACK_DIR}/package.json" > "${PACK_DIR}/package.json.tmp"
mv "${PACK_DIR}/package.json.tmp" "${PACK_DIR}/package.json"

# 安装依赖并打包
(cd "${PACK_DIR}" && \
    NODE_OPTIONS=--no-deprecation \
    npm install --ignore-scripts --no-audit --no-fund && \
    npm run compile && \
    npx vsce package --no-yarn && \
    mv *.vsix "${DIST_DIR}/rustdown-formatter.vsix") || exit 1

echo "打包完成！通用插件包位于: ${DIST_DIR}/rustdown-formatter.vsix"

# 清理临时目录
rm -rf "${PACK_DIR}"
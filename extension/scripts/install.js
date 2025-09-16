const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const platform = process.platform;
const binDir = path.join(__dirname, '..', 'bin');

// 确保 bin 目录存在
if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir);
}

// 根据平台选择可执行文件
const exeName = platform === 'win32' ? 'rustdown-formatter.exe' : 'rustdown-formatter';
const source = path.join(__dirname, '..', '..', 'target', 'release', exeName);
const target = path.join(binDir, exeName);

// 复制可执行文件
try {
    fs.copyFileSync(source, target);
    // Windows 上设置可执行权限不是必需的
    if (platform !== 'win32') {
        fs.chmodSync(target, 0o755);
    }
    console.log(`Successfully installed ${exeName}`);
} catch (error) {
    console.error(`Failed to install ${exeName}:`, error);
    process.exit(1);
}
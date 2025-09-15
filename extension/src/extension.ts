import * as vscode from 'vscode';
import * as path from 'path';
import * as child_process from 'child_process';

export function activate(context: vscode.ExtensionContext) {
    // 注册格式化提供程序
    let formatter = vscode.languages.registerDocumentFormattingEditProvider('markdown', {
        provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
            // 获取配置
            const config = vscode.workspace.getConfiguration('rustdownFormatter');

            // 创建临时配置文件
            const formatterConfig = {
                space_between_zh_and_en: config.get('spaceBetweenZhAndEn'),
                space_between_zh_and_num: config.get('spaceBetweenZhAndNum'),
                format_math: config.get('formatMath'),
                format_code_block: config.get('formatCodeBlock'),
                space_between_code_and_text: config.get('spaceBetweenCodeAndText'),
                code_formatters: {
                    "rust": "rustfmt",
                    "javascript": "prettier",
                    // ... 其他格式化工具配置
                }
            };

            // 调用 Rust 格式化工具
            try {
                const formatterPath = context.asAbsolutePath(path.join('bin', 'rustdown-formatter'));
                const result = child_process.spawnSync(formatterPath, {
                    input: document.getText(),
                    encoding: 'utf-8'
                });

                if (result.error) {
                    vscode.window.showErrorMessage(`格式化失败: ${result.error.message}`);
                    return [];
                }

                return [vscode.TextEdit.replace(
                    new vscode.Range(
                        document.lineAt(0).range.start,
                        document.lineAt(document.lineCount - 1).range.end
                    ),
                    result.stdout
                )];
            } catch (error) {
                vscode.window.showErrorMessage(`格式化失败: ${error}`);
                return [];
            }
        }
    });

    // 注册命令
    let command = vscode.commands.registerCommand('rustdown-formatter.format', () => {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
            vscode.commands.executeCommand('editor.action.formatDocument');
        }
    });

    context.subscriptions.push(formatter, command);
}

export function deactivate() { }
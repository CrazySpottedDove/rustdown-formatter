import * as vscode from 'vscode';
import * as path from 'path';
import * as child_process from 'child_process';

export function activate(context: vscode.ExtensionContext) {
    let formatter = vscode.languages.registerDocumentFormattingEditProvider('markdown', {
        provideDocumentFormattingEdits(document: vscode.TextDocument): Promise<vscode.TextEdit[]> {
            return new Promise((resolve, reject) => {
                // 确保文件已保存
                if (document.isDirty) {
                    document.save().then(() => {
                        formatFile(document, context, resolve, reject);
                    });
                } else {
                    formatFile(document, context, resolve, reject);
                }
            });
        }
    });

    context.subscriptions.push(formatter);
}

function getFormatterPath(context: vscode.ExtensionContext): string {
    const platform = process.platform;
    const platformDir = platform === 'win32' ? 'win32' : 'linux';
    const ext = platform === 'win32' ? '.exe' : '';

    return path.join(
        context.extensionPath,
        'bin',
        platformDir,
        `rustdown-formatter${ext}`
    );
}

function formatFile(
    document: vscode.TextDocument,
    context: vscode.ExtensionContext,
    resolve: (value: vscode.TextEdit[]) => void,
    reject: (reason?: any) => void
) {
    const formatterPath = getFormatterPath(context);
    const config = vscode.workspace.getConfiguration('rustdown-formatter');
    const configStr = JSON.stringify(config)
    try {
        // 直接传递文件路径给格式化工具
        child_process.execFile(formatterPath, [document.fileName],{env:{...process.env, RUSTDOWN_CONFIG: configStr}}, (error, stdout, stderr) => {
            if (error) {
                vscode.window.showErrorMessage(`格式化失败: ${error.message}`);
                reject(error);
                return;
            }

            // 读取文件获取更新后的内容
            const formatted = document.getText();
            const fullRange = new vscode.Range(
                document.positionAt(0),
                document.positionAt(document.getText().length)
            );
            resolve([vscode.TextEdit.replace(fullRange, formatted)]);
        });
    } catch (error) {
        vscode.window.showErrorMessage(`格式化失败: ${error}`);
        reject(error);
    }
}

export function deactivate() { }
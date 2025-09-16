# RustDown-Formatter

RustDown-Formatter 是一款面向中文使用者的，使用 rust 编写的 markdown 格式化工具，拥有如下特点：
- 格式化性能较好。
- 可对 latex 代码进行格式化。目前只支持对美元符的检测。
- 可通过配置的方式调用其它代码格式化工具，从而格式化您的 markdown 文件中的代码块。
- 可以自动为中英文、中文和数字之间添加空格。
- 可以为行内公式、行内代码与正文之间添加空格。
- 对 markdown 本身元素的格式化功能较少，在引用块中不会进行格式化。

## 工作效果示例

Before:

````md
# RustDown Formatter 测试文档

这是一个用于测试**中英文混排**、数字123混排、以及空格处理的文档。
This is a test for English and中文mixed content.

## 数学公式测试

行内公式示例：这是$E = mc^2$的公式。

块级公式示例：

$$
\begin{align*}
f(0.0)&=1.00000\\
f(0.2)&=1.22140\\
f(0.4)&=1.49182\\
f(0.6)&=1.82212\\
f(0.8)&=2.22554\end{align*}
$$

## 代码块测试

```js
function formatFile(document, context, resolve, reject) {const formatterPath = getFormatterPath(context);try {
        // 直接传递文件路径给格式化工具
child_process.execFile(formatterPath, [document.fileName], (error, stdout, stderr) => {
            if (    error   ) {
                vscode.window.showErrorMessage(`格式化失败: ${error.message}`);
                        reject(error);
                return;
            }
            // 读取文件获取更新后的内容
                const formatted = document.getText();
                const fullRange = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
    resolve([vscode.TextEdit.replace(fullRange, formatted)]);
        });
    }
catch (error) {
        vscode.window.showErrorMessage(`格式化失败: ${error}`);
        reject(error);
}
}
```

## 行内代码测试

请使用`cargo build`命令进行编译。
````

After:


````md
# RustDown Formatter 测试文档

这是一个用于测试**中英文混排**、数字 123 混排、以及空格处理的文档。
This is a test for English and 中文 mixed content.

## 数学公式测试

行内公式示例：这是 $E = mc^2$ 的公式。

块级公式示例：

$$
\begin{align*}
  f(0.0)&=1.00000\\
  f(0.2)&=1.22140\\
  f(0.4)&=1.49182\\
  f(0.6)&=1.82212\\
  f(0.8)&=2.22554
\end{align*}
$$

## 代码块测试

```js
function formatFile(document, context, resolve, reject) {
  const formatterPath = getFormatterPath(context);
  try {
    // 直接传递文件路径给格式化工具
    child_process.execFile(
      formatterPath,
      [document.fileName],
      (error, stdout, stderr) => {
        if (error) {
          vscode.window.showErrorMessage(`格式化失败: ${error.message}`);
          reject(error);
          return;
        }
        // 读取文件获取更新后的内容
        const formatted = document.getText();
        const fullRange = new vscode.Range(
          document.positionAt(0),
          document.positionAt(document.getText().length),
        );
        resolve([vscode.TextEdit.replace(fullRange, formatted)]);
      },
    );
  } catch (error) {
    vscode.window.showErrorMessage(`格式化失败: ${error}`);
    reject(error);
  }
}
```

## 行内代码测试

请使用 `cargo build` 命令进行编译。
````
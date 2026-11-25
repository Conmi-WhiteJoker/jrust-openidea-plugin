# jrust OpenIDEA 插件

给 [OpenIDEA](https://github.com/l0ng-ai/openidea) 增加 jrust 语言支持：语法高亮、新建项目、按文件运行、DAP 调试。

功能范围：

| 能力 | 实现 |
| --- | --- |
| 高亮 | 注册 `.jrust` 扩展名，复用 CodeMirror 的 `Java` 模式（jrust 是 Java 风格语法） |
| 新建项目 | **工具 › jrust: 新建项目…** 选父目录、输入名称，调用 `jrust new` 生成 `main.jrust`、`main.rs` 与 `Cargo.toml`（可直接 `cargo run`），并在新窗口打开 |
| 新建文件 | **文件 › 新建 › 文件…** 提供 “jrust Class” 模板，类名取自文件名（`$NAME$`） |
| 运行 | 为项目内每个 `*.jrust` 生成运行配置：`jrust run <绝对路径>` |
| 调试 | 运行配置的 `debugger` 留空，IDE 从 args 里的 `.jrust` 文件推断语言，再启动 `jrust dap`（stdio）走 DAP |

暂不包含补全、LSP、跳转定义。

## 前置条件

- OpenIDEA ≥ 0.1.51（插件 API 1.15；本插件要求 `^1.10`）。
- Rust 工具链 + `wasm32-wasip2` 目标。
- `openidea-plugin` CLI（OpenIDEA 仓库的 `ide-host` crate，`cargo build --bin openidea-plugin`，或 release 附件）。
- `jrust` 可执行文件在 `PATH` 中，且包含 `jrust new` 子命令（在 jrust 仓库执行 `cargo install --path .`）。

## 构建

```sh
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
mkdir -p dist
cp target/wasm32-wasip2/release/jrust_openidea_plugin.wasm dist/plugin.wasm
```

仓库内的 SDK 依赖固定到 OpenIDEA 的 `v0.1.51` 标签。若本地已有 SDK 源码，可把 `Cargo.toml` 里的 git 依赖换成 `path`，例如：

```toml
openidea-plugin = { path = "../openidea/sdk/rust/openidea-plugin" }
```

## 安装

方式一（CLI）：

```sh
openidea-plugin link /path/to/jrust-openidea-plugin
```

方式二（界面）：OpenIDEA → 设置 › 插件 › 从磁盘安装，选择本目录的 `plugin.toml`。

安装时会请求两项必需权限：

- `project.read`：列出项目里的 `*.jrust` 文件。
- `process:jrust`：启动 `jrust`（运行与 `jrust dap` 调试）。IDE 会按可执行文件的绝对路径与内容哈希固定它，每次启动前重新校验；升级 jrust 后需要重新授权。

## 使用

1. **新建项目**：**工具 › jrust: 新建项目…**（也可在查找操作里搜 `jrust: 新建项目`），选父目录、输入项目名。插件执行 `jrust new` 生成 `main.jrust`、即时转译的 `main.rs` 和 `Cargo.toml`（可直接 `cargo run`），再走 IDE 的打开流程在新窗口打开。
2. 已有项目直接在 OpenIDEA 里打开；需要额外源码文件时用 **jrust Class** 模板新建（如 `Main.jrust`）。
3. 项目根目录需要有 `main.jrust` 或 `Cargo.toml`——这是运行配置的触发标记（见下文限制）。
4. 打开任意 `.jrust` 文件，编辑器应显示 Java 风格高亮。
5. 运行配置下拉框里会出现 `jrust: <相对路径>`：
   - **Run**：执行 `jrust run <文件>`，输出进入运行工具窗口。
   - **Debug**：启动 `jrust dap`，断点按 `.jrust` 行号生效，支持变量、调用栈、单步（由 jrust 自带的 DAP 适配器提供）。

## 限制

- **新建项目向导不读取插件**：`File › New Project` 的生成器目录编译在 `ide-scaffold` 内（`generators/*.toml` 以 `include_str!` 打包），插件 API 1.15 没有对应贡献点，向导里不会出现 “jrust 项目”。等效入口是 **工具 › jrust: 新建项目…**：插件调用 `jrust new <目录>` 生成骨架，再 `host::open_project` 打开。
- **运行配置的触发标记**：`[[contributes.runConfigurations]]` 的 `files` 只认项目根目录下的具体文件名，不支持通配符，因此声明为 `main.jrust` 与 `Cargo.toml`。项目根目录两者都没有时，不会列出运行配置。
- **高亮精度**：没有 jrust 的 tree-sitter 语法，高亮来自 CodeMirror 的 Java 模式，个别 Rust 风格写法（如 `&`、所有权标注）着色可能不精确。
- **`jrust dap` 的 launch 兼容**：OpenIDEA 会把运行配置原样转发到 DAP `launch` 请求（`program = "jrust"`、`args = ["run", "<文件>"]`）。jrust 侧已兼容两种形态：`program` 直接是 `.jrust` 文件，或在 `args` 中查找第一个 `.jrust` 文件。

## 目录结构

```
plugin.toml           插件清单（权限、激活、语言/命令/文件模板/运行配置贡献点）
languages/jrust.toml  语言定义：扩展名、注释、缩进、高亮模式、DAP 调试器
templates/Main.jrust  新建文件模板（类名 $NAME$、光标停在 $END$）
src/lib.rs            “新建项目”命令 + launch-configurations 导出（扫描 *.jrust 生成运行/调试配置）
Cargo.toml            wasm32-wasip2 cdylib + OpenIDEA SDK
```

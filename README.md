# DevTriage

DevTriage 将本地错误编译成可追溯、经过脱敏、可交给人或外部 AI 的调试上下文。

## 当前垂直切片

```bash
printf 'TypeError: boom\n at run (src/a.ts:4:2)\n' | cargo run -p devtriage-cli
cargo run -p devtriage-cli -- --json path/to/error.log
```

当前版本包含跨平台 Rust 核心、通用能力包、CLI 验证入口和桌面壳。桌面分析只处理用户粘贴的输入；它不读取项目、不读取 Git 或其他历史、不访问进程或端口，也不联网。它仅在用户明确选择并确认目录后保存该目录的规范路径和确认时间，用于最近项目候选；绝不保存日志、分析上下文或秘密。

## 桌面壳

桌面前端需要 Node.js 20 或更高版本。安装依赖并启动 Tauri 开发环境：

```bash
pnpm --dir crates/devtriage-desktop/frontend install
(cd crates/devtriage-desktop && ./frontend/node_modules/.bin/tauri dev)
```

粘贴日志并选择 **Analyze** 后，可打开 Detail 窗口，切换 compact、standard 和 detailed 输出预算。复制操作只复制经过脱敏的 AI context。

## 核心管线

`normalize → run capability packs → merge evidence → redact → fingerprint → compile`

所有证据保留来源和能力包标识。能力包失败会被隔离；未知输入仍走通用解析。

## 安全说明

脱敏只能降低泄漏风险，不能保证识别所有秘密。默认输出前仍应允许用户审阅。分析核心没有网络依赖；未来的规则更新必须与分析进程隔离。

## 验证

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

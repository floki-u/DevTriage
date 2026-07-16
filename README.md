# DevTriage

DevTriage 将本地错误编译成可追溯、经过脱敏、可交给人或外部 AI 的调试上下文。

## 当前垂直切片

```bash
printf 'TypeError: boom\n at run (src/a.ts:4:2)\n' | cargo run -p devtriage-cli
cargo run -p devtriage-cli -- --json path/to/error.log
```

当前版本只包含跨平台 Rust 核心、通用能力包和 CLI 验证入口。它不读取项目、Git、进程或端口，不保存历史，也不联网。

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

# DevTriage 引擎基础垂直切片 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个完全本地、可从标准输入运行的 DevTriage Rust 垂直切片：归一化日志、组合运行能力包、生成可追溯证据、脱敏、编译默认 AI 上下文，并通过 CLI 输出。

**Architecture:** 使用 Cargo workspace，将无平台依赖的 `devtriage-core` 与验证入口 `devtriage-cli` 分开。核心流程固定为 `normalize → detect/run packs → merge evidence → redact → fingerprint → compile`；首个内置能力包只实现通用错误与 `path:line:column` 提取，为后续生态能力包建立稳定接口。

**Tech Stack:** Rust stable（2024 edition）、Cargo workspace、Serde、Regex、thiserror、BLAKE3、Clap、Anyhow、serde_json。依赖使用稳定主版本约束，由提交的 `Cargo.lock` 固定实际版本。本计划不创建 Tauri/React 界面；桌面外壳使用独立计划。

---

## 范围边界

本计划交付一条可运行、可测试的核心垂直切片，包括：

- 问题现场和证据类型；
- 输入归一化和有界截断；
- 可组合能力包接口与失败隔离；
- 通用错误、堆栈位置和重复行提取；
- 证据合并、来源保留和稳定 ID；
- 基础秘密脱敏与脱敏统计；
- 稳定错误指纹；
- 约 500/1,500/4,000 token 的输出预算；
- 从 stdin 或文件运行的 CLI；
- 黄金样例、组合、失败与隐私测试。

以下内容属于后续独立计划：Tauri/React、菜单栏、项目确认、Git/进程/端口平台适配、SQLite 和记忆存储、完整归档加密、社区规则加载、具体语言深度能力包、矛盾证据关系解析，以及私钥、Cookie、手机号、内网域名等完整脱敏规则集。

超大日志的“保留开头、结尾和能力包命中区域”采样也留给后续流式输入计划；本垂直切片只验证 UTF-8 边界安全的固定上限截断。

公开发布方式和软件许可证不属于本计划；workspace 不预设许可证字段，避免提前作出商业与开源决策。

## 文件结构

```text
.
├── Cargo.toml                       # workspace 与共享依赖
├── Cargo.lock                       # 固定解析后的依赖版本
├── rust-toolchain.toml              # Rust stable、rustfmt、clippy
├── .gitignore                       # 忽略构建产物与讨论画布
├── crates/
│   ├── devtriage-core/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs              # 对外导出稳定 API
│   │   │   ├── model.rs            # 问题现场、证据、来源与输出类型
│   │   │   ├── normalize.rs        # 文本归一化与有界截断
│   │   │   ├── pack.rs             # 能力包接口、注册中心与失败隔离
│   │   │   ├── graph.rs            # 证据合并和稳定 ID
│   │   │   ├── universal.rs        # 首个通用能力包
│   │   │   ├── redact.rs           # 本地脱敏与分类统计
│   │   │   ├── fingerprint.rs      # 脱敏后稳定错误指纹
│   │   │   ├── compiler.rs         # AI 上下文编译和预算裁剪
│   │   │   └── pipeline.rs         # 端到端编排
│   │   └── tests/
│   │       ├── pipeline.rs          # 端到端和组合测试
│   │       └── fixtures/
│   │           ├── js-error.log
│   │           └── mixed-secrets.log
│   └── devtriage-cli/
│       ├── Cargo.toml
│       ├── src/main.rs              # stdin/file → prompt/JSON
│       └── tests/cli.rs             # CLI 行为测试
└── README.md                        # 本地运行和架构边界
```

### Task 1: 初始化 Rust workspace

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: `crates/devtriage-core/Cargo.toml`
- Create: `crates/devtriage-core/src/lib.rs`
- Create: `crates/devtriage-cli/Cargo.toml`
- Create: `crates/devtriage-cli/src/main.rs`

- [ ] **Step 1: 安装并验证工具链**

先运行：

```bash
rustc --version
cargo --version
```

当前预期结果是 `command not found`。如果仍未安装，使用官方 rustup：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
```

再次运行 `rustc --version && cargo --version`，预期两个命令成功并显示 stable 工具链版本。

- [ ] **Step 2: 初始化 Git 和 workspace 文件**

运行：

```bash
git init
```

创建 `Cargo.toml`：

```toml
[workspace]
members = ["crates/devtriage-core", "crates/devtriage-cli"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"

[workspace.dependencies]
anyhow = "1"
blake3 = "1"
clap = { version = "4", features = ["derive"] }
regex = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

创建 `rust-toolchain.toml`：

```toml
[toolchain]
channel = "stable"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

创建 `.gitignore`：

```gitignore
/target/
/.superpowers/
.DS_Store
```

创建 `crates/devtriage-core/Cargo.toml`：

```toml
[package]
name = "devtriage-core"
version.workspace = true
edition.workspace = true

[dependencies]
blake3.workspace = true
regex.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
```

创建 `crates/devtriage-cli/Cargo.toml`：

```toml
[package]
name = "devtriage-cli"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
clap.workspace = true
devtriage-core = { path = "../devtriage-core" }
serde_json.workspace = true

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

创建最小 `crates/devtriage-core/src/lib.rs`：

```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

创建最小 `crates/devtriage-cli/src/main.rs`：

```rust
fn main() {
    println!("devtriage {}", devtriage_core::version());
}
```

- [ ] **Step 3: 格式化并验证 workspace**

运行：

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

预期：三个命令成功；Cargo 生成 `Cargo.lock`；测试结果包含两个 crate 且无失败。

- [ ] **Step 4: 提交初始化**

```bash
git add .gitignore Cargo.toml Cargo.lock rust-toolchain.toml crates docs
git commit -m "chore: initialize devtriage rust workspace"
```

### Task 2: 定义稳定的问题现场模型

**Files:**
- Create: `crates/devtriage-core/src/model.rs`
- Modify: `crates/devtriage-core/src/lib.rs`
- Create: `crates/devtriage-core/tests/model.rs`

- [ ] **Step 1: 先写模型序列化测试**

创建 `crates/devtriage-core/tests/model.rs`：

```rust
use devtriage_core::model::{AnalysisDepth, EvidenceKind, SCHEMA_VERSION};

#[test]
fn model_has_stable_external_names() {
    assert_eq!(SCHEMA_VERSION, 1);
    assert_eq!(serde_json::to_string(&AnalysisDepth::Structured).unwrap(), "\"structured\"");
    assert!(EvidenceKind::Error < EvidenceKind::StackFrame);
}
```

运行：

```bash
cargo test -p devtriage-core --test model
```

预期：编译失败，提示 `devtriage_core::model` 不存在。

- [ ] **Step 2: 实现问题现场模型**

创建 `crates/devtriage-core/src/model.rs`：

```rust
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisDepth {
    Generic,
    Structured,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Error,
    StackFrame,
    FilePath,
    Runtime,
    Project,
    LogExcerpt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Public,
    Sensitive,
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source_id: String,
    pub range: Option<SourceRange>,
    pub capability_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceDraft {
    pub kind: EvidenceKind,
    pub value: String,
    pub confidence: u8,
    pub sensitivity: Sensitivity,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: u64,
    pub kind: EvidenceKind,
    pub value: String,
    pub confidence: u8,
    pub sensitivity: Sensitivity,
    pub provenance: Vec<Provenance>,
    pub related: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transformation {
    pub kind: String,
    pub detail: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledOutput {
    pub text: String,
    pub estimated_tokens: usize,
    pub omitted_evidence: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueContext {
    pub schema_version: u16,
    pub analysis_depth: AnalysisDepth,
    pub evidence: Vec<Evidence>,
    pub transformations: Vec<Transformation>,
    pub fingerprint: String,
    pub output: CompiledOutput,
}
```

将 `crates/devtriage-core/src/lib.rs` 改为：

```rust
pub mod model;

pub use model::{AnalysisDepth, EvidenceKind, IssueContext};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 3: 验证模型测试通过**

运行：

```bash
cargo test -p devtriage-core --test model
```

预期：模型测试通过，JSON 名称为 `structured`，schema 版本为 1。

- [ ] **Step 4: 提交模型**

```bash
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/model.rs crates/devtriage-core/tests/model.rs
git commit -m "feat(core): define issue context model"
```

### Task 3: 实现输入归一化与有界截断

**Files:**
- Create: `crates/devtriage-core/src/normalize.rs`
- Modify: `crates/devtriage-core/src/lib.rs`

- [ ] **Step 1: 写归一化失败测试**

创建 `crates/devtriage-core/src/normalize.rs`：

```rust
use crate::model::Transformation;
use regex::Regex;
use std::sync::OnceLock;

pub const MAX_INPUT_BYTES: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInput {
    pub text: String,
    pub transformations: Vec<Transformation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_and_normalizes_newlines() {
        let result = normalize("\u{1b}[31mError\u{1b}[0m\r\nnext\rline");
        assert_eq!(result.text, "Error\nnext\nline");
        assert_eq!(result.transformations[0].kind, "ansi_removed");
    }

    #[test]
    fn truncates_on_utf8_boundary() {
        let input = "界".repeat(MAX_INPUT_BYTES);
        let result = normalize(&input);
        assert!(result.text.len() <= MAX_INPUT_BYTES);
        assert!(result.text.is_char_boundary(result.text.len()));
        assert!(result.transformations.iter().any(|item| item.kind == "input_truncated"));
    }
}
```

将 `pub mod normalize;` 加入 `lib.rs`，运行：

```bash
cargo test -p devtriage-core normalize::tests
```

预期：编译失败，提示找不到 `normalize` 函数。

- [ ] **Step 2: 实现最小归一化逻辑**

在 `normalize.rs` 的测试模块之前加入：

```rust
fn ansi_regex() -> &'static Regex {
    static ANSI: OnceLock<Regex> = OnceLock::new();
    ANSI.get_or_init(|| Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap())
}

pub fn normalize(input: &str) -> NormalizedInput {
    let ansi_count = ansi_regex().find_iter(input).count();
    let without_ansi = ansi_regex().replace_all(input, "");
    let mut text = without_ansi.replace("\r\n", "\n").replace('\r', "\n");
    let mut transformations = Vec::new();

    if ansi_count > 0 {
        transformations.push(Transformation {
            kind: "ansi_removed".into(),
            detail: "Removed terminal control sequences".into(),
            count: ansi_count,
        });
    }

    if text.len() > MAX_INPUT_BYTES {
        let mut boundary = MAX_INPUT_BYTES;
        while !text.is_char_boundary(boundary) {
            boundary -= 1;
        }
        text.truncate(boundary);
        transformations.push(Transformation {
            kind: "input_truncated".into(),
            detail: format!("Input limited to {MAX_INPUT_BYTES} bytes"),
            count: 1,
        });
    }

    NormalizedInput { text, transformations }
}
```

- [ ] **Step 3: 验证归一化测试与静态检查**

```bash
cargo test -p devtriage-core normalize::tests
cargo clippy -p devtriage-core --all-targets -- -D warnings
```

预期：2 个归一化测试通过，Clippy 无警告。

- [ ] **Step 4: 提交归一化器**

```bash
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/normalize.rs
git commit -m "feat(core): normalize bounded log input"
```

### Task 4: 建立能力包接口与失败隔离

**Files:**
- Create: `crates/devtriage-core/src/pack.rs`
- Modify: `crates/devtriage-core/src/lib.rs`

- [ ] **Step 1: 写组合和失败隔离测试**

创建 `crates/devtriage-core/src/pack.rs`，先定义测试所需接口和测试：

```rust
use crate::model::{AnalysisDepth, EvidenceDraft, Transformation};
use crate::normalize::NormalizedInput;
use std::panic::{AssertUnwindSafe, catch_unwind};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackDescriptor {
    pub id: &'static str,
    pub depth: AnalysisDepth,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackOutput {
    pub evidence: Vec<EvidenceDraft>,
    pub transformations: Vec<Transformation>,
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("analysis failed: {0}")]
    Analysis(String),
}

pub trait CapabilityPack: Send + Sync {
    fn descriptor(&self) -> PackDescriptor;
    fn detect(&self, input: &NormalizedInput) -> u8;
    fn analyze(&self, input: &NormalizedInput) -> Result<PackOutput, PackError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFailure {
    pub capability_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryOutput {
    pub outputs: Vec<(PackDescriptor, PackOutput)>,
    pub failures: Vec<PackFailure>,
}

#[derive(Default)]
pub struct CapabilityRegistry {
    packs: Vec<Box<dyn CapabilityPack>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct GoodPack;
    struct PanicPack;

    impl CapabilityPack for GoodPack {
        fn descriptor(&self) -> PackDescriptor {
            PackDescriptor { id: "good", depth: AnalysisDepth::Structured }
        }
        fn detect(&self, _: &NormalizedInput) -> u8 { 100 }
        fn analyze(&self, _: &NormalizedInput) -> Result<PackOutput, PackError> {
            Ok(PackOutput::default())
        }
    }

    impl CapabilityPack for PanicPack {
        fn descriptor(&self) -> PackDescriptor {
            PackDescriptor { id: "panic", depth: AnalysisDepth::Deep }
        }
        fn detect(&self, _: &NormalizedInput) -> u8 { 100 }
        fn analyze(&self, _: &NormalizedInput) -> Result<PackOutput, PackError> {
            panic!("broken pack")
        }
    }

    #[test]
    fn keeps_successful_output_when_another_pack_panics() {
        let mut registry = CapabilityRegistry::default();
        registry.register(GoodPack);
        registry.register(PanicPack);
        let input = NormalizedInput { text: "error".into(), transformations: vec![] };
        let output = registry.run(&input);
        assert_eq!(output.outputs.len(), 1);
        assert_eq!(output.failures[0].capability_id, "panic");
    }
}
```

将 `pub mod pack;` 加入 `lib.rs`。运行：

```bash
cargo test -p devtriage-core pack::tests
```

预期：编译失败，提示 `register` 和 `run` 不存在。

- [ ] **Step 2: 实现注册与隔离运行**

在 `CapabilityRegistry` 后、测试模块前加入：

```rust
impl CapabilityRegistry {
    pub fn register(&mut self, pack: impl CapabilityPack + 'static) {
        self.packs.push(Box::new(pack));
    }

    pub fn run(&self, input: &NormalizedInput) -> RegistryOutput {
        let mut result = RegistryOutput::default();

        for pack in &self.packs {
            if pack.detect(input) == 0 {
                continue;
            }
            let descriptor = pack.descriptor();
            match catch_unwind(AssertUnwindSafe(|| pack.analyze(input))) {
                Ok(Ok(output)) => result.outputs.push((descriptor, output)),
                Ok(Err(error)) => result.failures.push(PackFailure {
                    capability_id: descriptor.id.into(),
                    message: error.to_string(),
                }),
                Err(_) => result.failures.push(PackFailure {
                    capability_id: descriptor.id.into(),
                    message: "capability panicked".into(),
                }),
            }
        }

        result
    }
}
```

- [ ] **Step 3: 运行测试与 Clippy**

```bash
cargo test -p devtriage-core pack::tests
cargo clippy -p devtriage-core --all-targets -- -D warnings
```

预期：失败隔离测试通过；panic 信息可能出现在测试 stderr，但测试进程成功退出。

- [ ] **Step 4: 提交能力包协议**

```bash
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/pack.rs
git commit -m "feat(core): add composable capability registry"
```

### Task 5: 合并证据并保留来源

**Files:**
- Create: `crates/devtriage-core/src/graph.rs`
- Modify: `crates/devtriage-core/src/lib.rs`

- [ ] **Step 1: 写去重与来源合并测试**

创建 `crates/devtriage-core/src/graph.rs`：

```rust
use crate::model::{Evidence, EvidenceDraft};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EvidenceKind, Provenance, Sensitivity};

    fn draft(capability_id: &str, confidence: u8) -> EvidenceDraft {
        EvidenceDraft {
            kind: EvidenceKind::Error,
            value: " TypeError: boom ".into(),
            confidence,
            sensitivity: Sensitivity::Public,
            provenance: Provenance {
                source_id: "clipboard".into(),
                range: None,
                capability_id: capability_id.into(),
            },
        }
    }

    #[test]
    fn merges_equivalent_evidence_and_keeps_both_sources() {
        let graph = merge(vec![draft("generic", 70), draft("javascript", 95)]);
        assert_eq!(graph.len(), 1);
        assert_eq!(graph[0].id, 1);
        assert_eq!(graph[0].confidence, 95);
        assert_eq!(graph[0].provenance.len(), 2);
        assert_eq!(graph[0].value, "TypeError: boom");
    }
}
```

将 `pub mod graph;` 加入 `lib.rs`，运行：

```bash
cargo test -p devtriage-core graph::tests
```

预期：编译失败，提示 `merge` 不存在。

- [ ] **Step 2: 实现确定性合并**

在 `graph.rs` 的测试模块之前加入：

```rust
pub fn merge(drafts: Vec<EvidenceDraft>) -> Vec<Evidence> {
    let mut grouped: BTreeMap<(crate::model::EvidenceKind, String), Evidence> = BTreeMap::new();

    for draft in drafts {
        let normalized = draft.value.trim().split_whitespace().collect::<Vec<_>>().join(" ");
        let key = (draft.kind, normalized.to_lowercase());
        let next_id = grouped.len() as u64 + 1;
        let entry = grouped.entry(key).or_insert_with(|| Evidence {
            id: next_id,
            kind: draft.kind,
            value: normalized,
            confidence: draft.confidence,
            sensitivity: draft.sensitivity,
            provenance: Vec::new(),
            related: Vec::new(),
        });
        entry.confidence = entry.confidence.max(draft.confidence);
        entry.sensitivity = entry.sensitivity.max(draft.sensitivity);
        if !entry.provenance.contains(&draft.provenance) {
            entry.provenance.push(draft.provenance);
        }
    }

    grouped.into_values().enumerate().map(|(index, mut item)| {
        item.id = index as u64 + 1;
        item
    }).collect()
}
```

- [ ] **Step 3: 验证确定性**

```bash
cargo test -p devtriage-core graph::tests
cargo test -p devtriage-core graph::tests -- --test-threads=1
```

预期：两次运行均通过，生成的 ID 和顺序稳定。

- [ ] **Step 4: 提交证据合并**

```bash
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/graph.rs
git commit -m "feat(core): merge evidence with provenance"
```

### Task 6: 实现通用能力包

**Files:**
- Create: `crates/devtriage-core/src/universal.rs`
- Modify: `crates/devtriage-core/src/lib.rs`

- [ ] **Step 1: 写错误、位置和重复行测试**

创建 `crates/devtriage-core/src/universal.rs`，先写测试：

```rust
use crate::model::{AnalysisDepth, EvidenceDraft, EvidenceKind, Provenance, Sensitivity, SourceRange, Transformation};
use crate::normalize::NormalizedInput;
use crate::pack::{CapabilityPack, PackDescriptor, PackError, PackOutput};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub struct UniversalPack;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_primary_error_and_source_location() {
        let input = NormalizedInput {
            text: "TypeError: cannot read value\n  at render (src/UserList.tsx:42:7)".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert!(output.evidence.iter().any(|e| e.kind == EvidenceKind::Error));
        assert!(output.evidence.iter().any(|e| e.kind == EvidenceKind::StackFrame && e.value == "src/UserList.tsx:42:7"));
    }

    #[test]
    fn reports_duplicate_lines() {
        let input = NormalizedInput {
            text: "warning: retry\nwarning: retry\nwarning: retry".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert_eq!(output.transformations[0].kind, "duplicate_lines_collapsed");
        assert_eq!(output.transformations[0].count, 2);
    }

    #[test]
    fn unknown_text_still_produces_a_generic_excerpt() {
        let input = NormalizedInput {
            text: "unexpected state while starting worker".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert!(output.evidence.iter().any(|e| e.kind == EvidenceKind::LogExcerpt));
    }
}
```

将 `pub mod universal;` 加入 `lib.rs`，运行：

```bash
cargo test -p devtriage-core universal::tests
```

预期：编译失败，因为 `UniversalPack` 尚未实现 `CapabilityPack`。

- [ ] **Step 2: 实现通用提取**

在 `universal.rs` 的测试模块之前加入：

```rust
fn location_regex() -> &'static Regex {
    static LOCATION: OnceLock<Regex> = OnceLock::new();
    LOCATION.get_or_init(|| {
        Regex::new(r"(?P<path>(?:[A-Za-z]:)?[A-Za-z0-9_./\\-]+\.[A-Za-z0-9]+):(?P<line>\d+)(?::(?P<column>\d+))?").unwrap()
    })
}

fn looks_like_error(line: &str) -> bool {
    let lower = line.to_lowercase();
    ["error", "exception", "panic", "fatal", "failed"]
        .iter()
        .any(|marker| lower.contains(marker))
}

impl CapabilityPack for UniversalPack {
    fn descriptor(&self) -> PackDescriptor {
        PackDescriptor { id: "official.universal", depth: AnalysisDepth::Generic }
    }

    fn detect(&self, _: &NormalizedInput) -> u8 {
        1
    }

    fn analyze(&self, input: &NormalizedInput) -> Result<PackOutput, PackError> {
        let mut evidence = Vec::new();
        let mut counts = BTreeMap::<&str, usize>::new();

        for line in input.text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            *counts.entry(line).or_default() += 1;
        }

        if let Some((offset, line)) = input.text.lines().scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        }).find(|(_, line)| looks_like_error(line)) {
            evidence.push(EvidenceDraft {
                kind: EvidenceKind::Error,
                value: line.trim().into(),
                confidence: 70,
                sensitivity: Sensitivity::Public,
                provenance: Provenance {
                    source_id: "normalized_input".into(),
                    range: Some(SourceRange { start: offset, end: offset + line.len() }),
                    capability_id: "official.universal".into(),
                },
            });
        }

        if !evidence.iter().any(|item| item.kind == EvidenceKind::Error) {
            if let Some(line) = input.text.lines().map(str::trim).find(|line| !line.is_empty()) {
                evidence.push(EvidenceDraft {
                    kind: EvidenceKind::LogExcerpt,
                    value: line.into(),
                    confidence: 30,
                    sensitivity: Sensitivity::Public,
                    provenance: Provenance {
                        source_id: "normalized_input".into(),
                        range: None,
                        capability_id: "official.universal".into(),
                    },
                });
            }
        }

        for captures in location_regex().captures_iter(&input.text) {
            let whole = captures.get(0).unwrap();
            evidence.push(EvidenceDraft {
                kind: EvidenceKind::StackFrame,
                value: whole.as_str().into(),
                confidence: 80,
                sensitivity: Sensitivity::Sensitive,
                provenance: Provenance {
                    source_id: "normalized_input".into(),
                    range: Some(SourceRange { start: whole.start(), end: whole.end() }),
                    capability_id: "official.universal".into(),
                },
            });
        }

        let duplicates = counts.values().map(|count| count.saturating_sub(1)).sum();
        let transformations = if duplicates > 0 {
            vec![Transformation {
                kind: "duplicate_lines_collapsed".into(),
                detail: "Repeated identical lines were represented once".into(),
                count: duplicates,
            }]
        } else {
            Vec::new()
        };

        Ok(PackOutput { evidence, transformations })
    }
}
```

- [ ] **Step 3: 运行测试并检查误匹配**

```bash
cargo test -p devtriage-core universal::tests
cargo clippy -p devtriage-core --all-targets -- -D warnings
```

预期：3 个测试通过，Clippy 无警告。

- [ ] **Step 4: 提交通用能力包**

```bash
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/universal.rs
git commit -m "feat(core): add universal error capability"
```

### Task 7: 实现脱敏与稳定错误指纹

**Files:**
- Create: `crates/devtriage-core/src/redact.rs`
- Create: `crates/devtriage-core/src/fingerprint.rs`
- Modify: `crates/devtriage-core/src/lib.rs`

- [ ] **Step 1: 写秘密不泄漏测试**

创建 `crates/devtriage-core/src/redact.rs`：

```rust
use crate::model::{Evidence, Transformation};
use regex::{Captures, Regex};
use std::sync::OnceLock;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EvidenceKind, Provenance, Sensitivity};

    #[test]
    fn redacts_assignments_jwts_and_emails_without_revealing_values() {
        let mut evidence = vec![Evidence {
            id: 1,
            kind: EvidenceKind::Error,
            value: "token=supersecret user=a@example.com jwt=eyJabc.def.ghi".into(),
            confidence: 90,
            sensitivity: Sensitivity::Public,
            provenance: vec![Provenance { source_id: "clipboard".into(), range: None, capability_id: "test".into() }],
            related: vec![],
        }];
        let transformations = redact_evidence(&mut evidence);
        assert!(!evidence[0].value.contains("supersecret"));
        assert!(!evidence[0].value.contains("a@example.com"));
        assert!(!evidence[0].value.contains("eyJabc"));
        assert_eq!(transformations.iter().map(|item| item.count).sum::<usize>(), 3);
        assert_eq!(evidence[0].sensitivity, Sensitivity::Secret);
    }
}
```

将 `pub mod redact;` 加入 `lib.rs`，运行：

```bash
cargo test -p devtriage-core redact::tests
```

预期：编译失败，提示 `redact_evidence` 不存在。

- [ ] **Step 2: 实现三类基础脱敏**

在 `redact.rs` 的测试模块之前加入：

```rust
fn assignment_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"(?i)\b(token|secret|password|passwd|api[_-]?key)\s*[:=]\s*([^\s,;]+)").unwrap())
}

fn jwt_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b").unwrap())
}

fn email_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap())
}

pub fn redact_evidence(evidence: &mut [Evidence]) -> Vec<Transformation> {
    let mut assignment_count = 0usize;
    let mut jwt_count = 0usize;
    let mut email_count = 0usize;

    for item in evidence {
        let value = assignment_regex().replace_all(&item.value, |caps: &Captures<'_>| {
            assignment_count += 1;
            format!("{}=[REDACTED:CREDENTIAL]", &caps[1])
        });
        let value = jwt_regex().replace_all(&value, |_: &Captures<'_>| {
            jwt_count += 1;
            "[REDACTED:JWT]"
        });
        let value = email_regex().replace_all(&value, |_: &Captures<'_>| {
            email_count += 1;
            "[REDACTED:EMAIL]"
        });
        if value != item.value {
            item.value = value.into_owned();
            item.sensitivity = crate::model::Sensitivity::Secret;
        }
    }

    [
        ("credential_redacted", "Credential-like values redacted", assignment_count),
        ("jwt_redacted", "JWT-like values redacted", jwt_count),
        ("email_redacted", "Email addresses redacted", email_count),
    ].into_iter().filter(|(_, _, count)| *count > 0).map(|(kind, detail, count)| Transformation {
        kind: kind.into(), detail: detail.into(), count,
    }).collect()
}
```

- [ ] **Step 3: 写并实现指纹测试**

创建 `crates/devtriage-core/src/fingerprint.rs`：

```rust
use crate::model::{Evidence, EvidenceKind};

pub fn fingerprint(evidence: &[Evidence]) -> String {
    let canonical = evidence.iter()
        .filter(|item| matches!(item.kind, EvidenceKind::Error | EvidenceKind::StackFrame | EvidenceKind::LogExcerpt))
        .map(|item| format!("{:?}:{}", item.kind, item.value.to_lowercase()))
        .collect::<Vec<_>>()
        .join("\n");
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Provenance, Sensitivity};

    fn item(value: &str) -> Evidence {
        Evidence {
            id: 1,
            kind: EvidenceKind::Error,
            value: value.into(),
            confidence: 90,
            sensitivity: Sensitivity::Public,
            provenance: vec![Provenance { source_id: "clipboard".into(), range: None, capability_id: "test".into() }],
            related: vec![],
        }
    }

    #[test]
    fn fingerprint_is_stable_and_value_sensitive() {
        assert_eq!(fingerprint(&[item("Boom")]), fingerprint(&[item("boom")]));
        assert_ne!(fingerprint(&[item("Boom")]), fingerprint(&[item("Other")]));
    }
}
```

将 `pub mod fingerprint;` 加入 `lib.rs`，运行：

```bash
cargo test -p devtriage-core redact::tests
cargo test -p devtriage-core fingerprint::tests
```

预期：脱敏和指纹测试全部通过。

- [ ] **Step 4: 运行安全回归与提交**

```bash
cargo test -p devtriage-core
cargo clippy -p devtriage-core --all-targets -- -D warnings
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/redact.rs crates/devtriage-core/src/fingerprint.rs
git commit -m "feat(core): redact secrets and fingerprint errors"
```

### Task 8: 编译有预算的 AI 上下文

**Files:**
- Create: `crates/devtriage-core/src/compiler.rs`
- Modify: `crates/devtriage-core/src/lib.rs`

- [ ] **Step 1: 写默认问题和预算测试**

创建 `crates/devtriage-core/src/compiler.rs`：

```rust
use crate::model::{CompiledOutput, Evidence, EvidenceKind, Transformation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputBudget {
    Compact,
    Standard,
    Detailed,
}

impl OutputBudget {
    fn token_limit(self) -> usize {
        match self {
            Self::Compact => 500,
            Self::Standard => 1_500,
            Self::Detailed => 4_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Provenance, Sensitivity};

    fn evidence(kind: EvidenceKind, value: &str) -> Evidence {
        Evidence {
            id: 1,
            kind,
            value: value.into(),
            confidence: 90,
            sensitivity: Sensitivity::Public,
            provenance: vec![Provenance { source_id: "clipboard".into(), range: None, capability_id: "test".into() }],
            related: vec![],
        }
    }

    #[test]
    fn standard_output_separates_facts_from_request() {
        let output = compile(
            &[evidence(EvidenceKind::Error, "TypeError: boom"), evidence(EvidenceKind::StackFrame, "src/a.ts:4:2")],
            &[],
            OutputBudget::Standard,
        );
        assert!(output.text.contains("## Facts"));
        assert!(output.text.contains("## Request"));
        assert!(output.text.contains("most likely cause"));
    }

    #[test]
    fn compact_output_respects_estimated_budget() {
        let long = "x".repeat(10_000);
        let output = compile(&[evidence(EvidenceKind::Error, &long)], &[], OutputBudget::Compact);
        assert!(output.estimated_tokens <= 500);
        assert_eq!(output.omitted_evidence, 1);
    }
}
```

将 `pub mod compiler;` 加入 `lib.rs`，运行：

```bash
cargo test -p devtriage-core compiler::tests
```

预期：编译失败，提示 `compile` 不存在。

- [ ] **Step 2: 实现优先级与预算裁剪**

在 `compiler.rs` 的测试模块之前加入：

```rust
fn priority(kind: EvidenceKind) -> u8 {
    match kind {
        EvidenceKind::Error => 0,
        EvidenceKind::StackFrame => 1,
        EvidenceKind::Runtime => 2,
        EvidenceKind::Project => 3,
        EvidenceKind::FilePath => 4,
        EvidenceKind::LogExcerpt => 5,
    }
}

fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

pub fn compile(
    evidence: &[Evidence],
    transformations: &[Transformation],
    budget: OutputBudget,
) -> CompiledOutput {
    let mut ordered = evidence.to_vec();
    ordered.sort_by_key(|item| (priority(item.kind), std::cmp::Reverse(item.confidence), item.id));

    let request = "## Request\nIdentify the most likely cause using only the facts above, then recommend the next diagnostic or repair step.";
    let mut text = String::from("## Facts\n");
    let mut omitted = 0usize;
    let max_chars = budget.token_limit() * 4;

    for item in ordered {
        let line = format!("- {:?}: {}\n", item.kind, item.value);
        if text.len() + line.len() + request.len() + 2 <= max_chars {
            text.push_str(&line);
        } else {
            omitted += 1;
        }
    }

    if !transformations.is_empty() {
        let summary = transformations.iter()
            .map(|item| format!("{}={}", item.kind, item.count))
            .collect::<Vec<_>>()
            .join(", ");
        let line = format!("\n## Transformations\n{summary}\n");
        if text.len() + line.len() + request.len() + 2 <= max_chars {
            text.push_str(&line);
        }
    }

    text.push('\n');
    text.push_str(request);
    if text.len() > max_chars {
        text.truncate(max_chars);
    }

    CompiledOutput {
        estimated_tokens: estimate_tokens(&text),
        text,
        omitted_evidence: omitted,
    }
}
```

- [ ] **Step 3: 运行预算测试**

```bash
cargo test -p devtriage-core compiler::tests
cargo clippy -p devtriage-core --all-targets -- -D warnings
```

预期：2 个测试通过，标准输出包含事实与请求两个分区，精简输出不超过估算预算。

- [ ] **Step 4: 提交上下文编译器**

```bash
git add crates/devtriage-core/src/lib.rs crates/devtriage-core/src/compiler.rs
git commit -m "feat(core): compile budgeted ai context"
```

### Task 9: 组装端到端管线

**Files:**
- Create: `crates/devtriage-core/src/pipeline.rs`
- Modify: `crates/devtriage-core/src/lib.rs`
- Create: `crates/devtriage-core/tests/pipeline.rs`
- Create: `crates/devtriage-core/tests/fixtures/js-error.log`
- Create: `crates/devtriage-core/tests/fixtures/mixed-secrets.log`

- [ ] **Step 1: 创建黄金输入样例**

创建 `crates/devtriage-core/tests/fixtures/js-error.log`：

```text
TypeError: Cannot read properties of undefined (reading 'name')
    at renderUser (src/pages/UserList.tsx:42:7)
    at renderUser (src/pages/UserList.tsx:42:7)
```

创建 `crates/devtriage-core/tests/fixtures/mixed-secrets.log`：

```text
Fatal error for user=developer@example.com
token=do-not-leak-this
    at request (src/api/client.ts:18:3)
```

- [ ] **Step 2: 写端到端失败测试**

创建 `crates/devtriage-core/tests/pipeline.rs`：

```rust
use devtriage_core::{OutputBudget, Pipeline};

#[test]
fn compiles_a_redacted_traceable_context() {
    let input = include_str!("fixtures/js-error.log");
    let context = Pipeline::default().analyze(input, OutputBudget::Standard);
    assert_eq!(context.schema_version, 1);
    assert!(context.output.text.contains("TypeError"));
    assert!(context.output.text.contains("src/pages/UserList.tsx:42:7"));
    assert!(context.transformations.iter().any(|item| item.kind == "duplicate_lines_collapsed"));
    assert_eq!(context.fingerprint.len(), 64);
}

#[test]
fn secrets_never_reach_compiled_output() {
    let input = include_str!("fixtures/mixed-secrets.log");
    let context = Pipeline::default().analyze(input, OutputBudget::Standard);
    assert!(!context.output.text.contains("do-not-leak-this"));
    assert!(!context.output.text.contains("developer@example.com"));
    assert!(context.output.text.contains("[REDACTED:CREDENTIAL]"));
}
```

在 `lib.rs` 中导出：

```rust
pub mod pipeline;

pub use compiler::OutputBudget;
pub use pipeline::Pipeline;
```

运行：

```bash
cargo test -p devtriage-core --test pipeline
```

预期：编译失败，提示 `Pipeline` 不存在。

- [ ] **Step 3: 实现管线编排**

创建 `crates/devtriage-core/src/pipeline.rs`：

```rust
use crate::compiler::{OutputBudget, compile};
use crate::fingerprint::fingerprint;
use crate::graph::merge;
use crate::model::{AnalysisDepth, IssueContext, SCHEMA_VERSION, Transformation};
use crate::normalize::normalize;
use crate::pack::CapabilityRegistry;
use crate::redact::redact_evidence;
use crate::universal::UniversalPack;

pub struct Pipeline {
    registry: CapabilityRegistry,
}

impl Default for Pipeline {
    fn default() -> Self {
        let mut registry = CapabilityRegistry::default();
        registry.register(UniversalPack);
        Self { registry }
    }
}

impl Pipeline {
    pub fn analyze(&self, raw: &str, budget: OutputBudget) -> IssueContext {
        let normalized = normalize(raw);
        let registry_output = self.registry.run(&normalized);
        let depth = registry_output.outputs.iter()
            .map(|(descriptor, _)| descriptor.depth)
            .max()
            .unwrap_or(AnalysisDepth::Generic);

        let mut transformations = normalized.transformations;
        let mut drafts = Vec::new();
        for (_, output) in registry_output.outputs {
            drafts.extend(output.evidence);
            transformations.extend(output.transformations);
        }
        transformations.extend(registry_output.failures.into_iter().map(|failure| Transformation {
            kind: "capability_failed".into(),
            detail: format!("{}: {}", failure.capability_id, failure.message),
            count: 1,
        }));

        let mut evidence = merge(drafts);
        transformations.extend(redact_evidence(&mut evidence));
        let fingerprint = fingerprint(&evidence);
        let output = compile(&evidence, &transformations, budget);

        IssueContext {
            schema_version: SCHEMA_VERSION,
            analysis_depth: depth,
            evidence,
            transformations,
            fingerprint,
            output,
        }
    }
}
```

- [ ] **Step 4: 验证端到端行为**

```bash
cargo test -p devtriage-core --test pipeline
cargo test -p devtriage-core
cargo clippy -p devtriage-core --all-targets -- -D warnings
```

预期：两个集成测试及全部单元测试通过，秘密原值不出现在输出中。

- [ ] **Step 5: 提交端到端管线**

```bash
git add crates/devtriage-core
git commit -m "feat(core): assemble local analysis pipeline"
```

### Task 10: 提供 CLI 验证入口

**Files:**
- Modify: `crates/devtriage-cli/src/main.rs`
- Create: `crates/devtriage-cli/tests/cli.rs`

- [ ] **Step 1: 写 stdin 和 JSON 模式测试**

创建 `crates/devtriage-cli/tests/cli.rs`：

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn reads_stdin_and_prints_ai_context() {
    let mut command = Command::cargo_bin("devtriage-cli").unwrap();
    command.write_stdin("TypeError: boom\n at run (src/a.ts:4:2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("## Facts"))
        .stdout(predicate::str::contains("## Request"));
}

#[test]
fn json_mode_never_prints_the_original_secret() {
    let mut command = Command::cargo_bin("devtriage-cli").unwrap();
    command.arg("--json")
        .write_stdin("fatal token=hidden-value")
        .assert()
        .success()
        .stdout(predicate::str::contains("credential_redacted"))
        .stdout(predicate::str::contains("hidden-value").not());
}
```

运行：

```bash
cargo test -p devtriage-cli --test cli
```

预期：测试失败；当前 CLI 不读取 stdin，也不支持 `--json`。

- [ ] **Step 2: 实现 CLI 参数与输入**

将 `crates/devtriage-cli/src/main.rs` 替换为：

```rust
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use devtriage_core::{OutputBudget, Pipeline};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BudgetArg {
    Compact,
    Standard,
    Detailed,
}

impl From<BudgetArg> for OutputBudget {
    fn from(value: BudgetArg) -> Self {
        match value {
            BudgetArg::Compact => Self::Compact,
            BudgetArg::Standard => Self::Standard,
            BudgetArg::Detailed => Self::Detailed,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "devtriage", about = "Compile local debug evidence for humans and AI")]
struct Args {
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    #[arg(long, value_enum, default_value_t = BudgetArg::Standard)]
    budget: BudgetArg,
}

fn read_input(file: Option<PathBuf>) -> Result<String> {
    match file {
        Some(path) => std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display())),
        None => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).context("failed to read stdin")?;
            Ok(input)
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = read_input(args.file)?;
    let context = Pipeline::default().analyze(&input, args.budget.into());
    if args.json {
        println!("{}", serde_json::to_string_pretty(&context)?);
    } else {
        println!("{}", context.output.text);
    }
    Ok(())
}
```

- [ ] **Step 3: 运行 CLI 测试和手动冒烟测试**

```bash
cargo test -p devtriage-cli --test cli
printf 'TypeError: boom\n at run (src/a.ts:4:2)\n' | cargo run -q -p devtriage-cli
printf 'fatal token=hidden-value\n' | cargo run -q -p devtriage-cli -- --json
```

预期：测试通过；第一个命令输出 `## Facts` 和 `## Request`；第二个命令包含 `[REDACTED:CREDENTIAL]`，且不包含 `hidden-value`。

- [ ] **Step 4: 提交 CLI**

```bash
git add crates/devtriage-cli
git commit -m "feat(cli): analyze logs from stdin or file"
```

### Task 11: 补齐文档与全量验证

**Files:**
- Create: `README.md`

- [ ] **Step 1: 写清当前能力和安全边界**

创建 `README.md`：

````markdown
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
````

- [ ] **Step 2: 运行格式化、测试和 lint**

```bash
cargo fmt
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

预期：所有命令成功，无测试失败或 Clippy 警告。

- [ ] **Step 3: 验证工作区只包含预期改动**

```bash
git status --short
git diff --check
```

预期：`git diff --check` 无输出；`git status --short` 只显示 `README.md`，以及格式化实际修改过的预期 Rust 文件。

- [ ] **Step 4: 提交文档与格式化结果**

```bash
git add README.md Cargo.lock crates
git commit -m "docs: document engine vertical slice"
```

- [ ] **Step 5: 最终验证提交状态**

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git status --short
git log --oneline -5
```

预期：测试与 Clippy 成功；工作区干净；最近提交按顺序展示 CLI、管线、编译器及文档等小步提交。

## 完成标准

- 同一输入在相同版本下产生稳定的证据顺序、错误指纹和 JSON schema。
- 通用错误与 `path:line:column` 能从样例中提取，并带有来源和能力包 ID。
- 多个能力包可同时运行；单个能力包返回错误或 panic 不会破坏其他结果。
- JWT、凭据赋值和邮箱样例不会出现在最终提示词或 JSON 输出中。
- 标准输出包含彼此分离的事实与请求；三档预算都不超过本地估算上限。
- CLI 可以从 stdin 或 UTF-8 文本文件运行，不读取项目、不保存数据、不访问网络。
- `cargo fmt --check`、`cargo test --workspace` 和严格 Clippy 全部通过。

# DevTriage Desktop Shell and Project Association Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a macOS Tauri 2 + React menu-bar application that analyzes pasted logs through `devtriage-core`, suggests projects, and copies redacted AI context.

**Architecture:** Add `crates/devtriage-desktop` to the Cargo workspace. Its Rust backend owns `Pipeline`, project-candidate ranking, recent-project persistence, and Tauri commands; its React frontend only renders command responses and in-memory UI state. A tray click opens the compact main window; a second singleton window shows the current `IssueContext` in greater detail.

**Tech Stack:** Rust stable 2024, Tauri 2.8, React 18, TypeScript, Vite 5, pnpm, Vitest, Testing Library, serde_json.

## Global Constraints

- Target macOS only; require Node.js 20+ and pnpm 9+ before scaffolding. The current Node 14 installation is insufficient.
- Use Tauri Rust commands; do not spawn `devtriage-cli`.
- Persist only canonical project paths and confirmation times under Tauri `app_data_dir`; never persist raw input, `IssueContext`, clipboard text, or secrets.
- Do not add project-file, Git, process, port, or network access.
- All copyable output must come from `IssueContext.output.text` returned by `Pipeline`.
- Pin Tauri packages to the 2.8 minor line and keep all capabilities least-privilege.

---

## File Structure

```text
Cargo.toml                                      # adds desktop workspace member
crates/devtriage-desktop/
  Cargo.toml                                    # Tauri backend and core dependency
  build.rs
  src/lib.rs                                    # app setup, tray and window lifecycle
  src/main.rs                                   # production entry point
  src/domain.rs                                 # ProjectCandidate, RecentProject, DesktopState
  src/projects.rs                               # pure path extraction and ranking
  src/recent.rs                                 # JSON-backed path/time repository
  src/commands.rs                               # invoke-facing request/response commands
  tests/projects.rs                             # ranking tests
  tests/recent.rs                               # persistence tests
  tauri.conf.json                               # windows, bundle identifier and frontend commands
  capabilities/default.json                     # only core/default capability
  frontend/
    package.json
    vite.config.ts
    src/api.ts                                  # typed invoke boundary
    src/App.tsx                                 # quick-panel workflow
    src/Detail.tsx                              # detail window
    src/main.tsx
    src/styles.css
    src/App.test.tsx
    src/Detail.test.tsx
```

### Task 1: Establish the desktop workspace and repeatable frontend toolchain

**Files:** Modify `Cargo.toml`, `.gitignore`; create all `crates/devtriage-desktop` scaffold/configuration files and `frontend/package.json`.

**Interfaces:** Produces `devtriage-desktop` with `cargo check -p devtriage-desktop` and `pnpm --dir crates/devtriage-desktop/frontend test` commands.

- [ ] **Step 1: Verify compatible local tools.**

Run `node --version`, `pnpm --version`, `rustc --version`. Stop and ask the user to install Node 20+ if Node reports less than 20; do not downgrade the frontend stack to retain Node 14.

- [ ] **Step 2: Write the failing Rust compile check.**

Add the workspace member before its manifest exists:

```toml
[workspace]
members = ["crates/devtriage-core", "crates/devtriage-cli", "crates/devtriage-desktop"]
resolver = "2"
```

Run `cargo check -p devtriage-desktop`. Expected: `package ID specification 'devtriage-desktop' did not match any packages`.

- [ ] **Step 3: Add the minimal desktop manifests and Tauri configuration.**

Create `crates/devtriage-desktop/Cargo.toml`:

```toml
[package]
name = "devtriage-desktop"
version.workspace = true
edition.workspace = true
build = "build.rs"

[build-dependencies]
tauri-build = "2.3"

[dependencies]
devtriage-core = { path = "../devtriage-core" }
serde.workspace = true
serde_json.workspace = true
tauri = { version = "2.8", features = ["tray-icon"] }
tauri-plugin-clipboard-manager = "2.3"
tauri-plugin-dialog = "2.3"
thiserror.workspace = true
```

Create `build.rs` with `fn main() { tauri_build::build() }`, create `src/main.rs` with `fn main() { devtriage_desktop::run() }`, and add a temporary `pub fn run() {}` in `src/lib.rs`.

Create `frontend/package.json` with scripts `dev: "vite"`, `build: "tsc -b && vite build"`, `test: "vitest run"` and dependencies `@tauri-apps/api: "2.8.x"`, `@tauri-apps/plugin-clipboard-manager: "2.3.x"`, `@tauri-apps/plugin-dialog: "2.3.x"`, `react: "18.3.x"`, `react-dom: "18.3.x"`; dev dependencies `typescript: "5.7.x"`, `vite: "5.4.x"`, `vitest: "2.1.x"`, `@testing-library/react: "16.1.x"`, `jsdom: "25.0.x"`.

Set `tauri.conf.json` `productName` to `DevTriage`, `identifier` to `com.floki.devtriage`, `build.beforeDevCommand` to `pnpm --dir frontend dev --host 127.0.0.1`, `build.beforeBuildCommand` to `pnpm --dir frontend build`, `build.devUrl` to `http://127.0.0.1:5173`, and one hidden `main` window sized `440x620`.

- [ ] **Step 4: Verify the scaffold.**

Run `pnpm --dir crates/devtriage-desktop/frontend install`, `cargo check -p devtriage-desktop`, and `pnpm --dir crates/devtriage-desktop/frontend test`. Expected: all succeed.

- [ ] **Step 5: Commit.**

```bash
git add Cargo.toml Cargo.lock .gitignore crates/devtriage-desktop
git commit -m "chore(desktop): scaffold tauri react app"
```

### Task 2: Build pure project candidate ranking and recent-project persistence

**Files:** Create `src/domain.rs`, `src/projects.rs`, `src/recent.rs`, `tests/projects.rs`, `tests/recent.rs`.

**Interfaces:** `rank_projects(input: &str, recent: &[RecentProject]) -> Vec<ProjectCandidate>` and `RecentProjectStore::{load,confirm}` are consumed by commands.

- [ ] **Step 1: Write failing ranking and persistence tests.**

```rust
#[test]
fn absolute_log_path_ranks_matching_recent_project_first() {
    let recent = vec![RecentProject::new("/work/app", 10)];
    let candidates = rank_projects("at run (/work/app/src/main.ts:4:2)", &recent);
    assert_eq!(candidates[0].path, "/work/app");
    assert_eq!(candidates[0].reason, CandidateReason::AbsolutePath);
}

#[test]
fn confirm_overwrites_path_and_keeps_only_timestamp_and_path() {
    let store = RecentProjectStore::at(tempfile::tempdir().unwrap().path());
    store.confirm("/work/app", 10).unwrap();
    assert_eq!(store.load().unwrap(), vec![RecentProject::new("/work/app", 10)]);
}
```

Run `cargo test -p devtriage-desktop --test projects --test recent`. Expected: fail because the types do not exist.

- [ ] **Step 2: Implement the pure domain and repository.**

Define serializable types:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject { pub path: String, pub confirmed_at: i64 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCandidate { pub path: String, pub score: u16, pub reason: CandidateReason }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateReason { AbsolutePath, RecentRelativePath, RecentProject }
```

Extract absolute path prefixes with a regex, canonicalize only user-confirmed paths using `std::fs::canonicalize`, and sort candidates by descending score then ascending path. `RecentProjectStore` writes a JSON array named `recent-projects.json` with `std::fs::write`; `confirm` replaces the same path, sorts by descending `confirmed_at`, and retains 10 records.

- [ ] **Step 3: Verify behavior and lint.**

Run `cargo test -p devtriage-desktop --test projects --test recent` and `cargo clippy -p devtriage-desktop --all-targets -- -D warnings`. Expected: pass.

- [ ] **Step 4: Commit.**

```bash
git add crates/devtriage-desktop/src crates/devtriage-desktop/tests
git commit -m "feat(desktop): rank and remember confirmed projects"
```

### Task 3: Expose safe analysis commands and in-memory current context

**Files:** Create `src/commands.rs`; modify `src/lib.rs`.

**Interfaces:** Commands are `analyze_input(input: String, budget: OutputBudget) -> AnalysisResponse`, `project_candidates(input: String) -> Vec<ProjectCandidate>`, `confirm_project(path: String) -> Result<(), CommandError>`, and `current_context(budget: OutputBudget) -> Option<IssueContext>`.

- [ ] **Step 1: Write failing command tests.**

```rust
#[test]
fn analysis_response_contains_redacted_context_and_candidates() {
    let state = DesktopState::for_test();
    let response = analyze_input(State(&state), "fatal token=hidden-value".into(), OutputBudget::Standard).unwrap();
    assert!(response.context.output.text.contains("[REDACTED:CREDENTIAL]"));
    assert!(!response.context.output.text.contains("hidden-value"));
}
```

Run `cargo test -p devtriage-desktop commands::tests`. Expected: compile failure for missing command and state.

- [ ] **Step 2: Implement `DesktopState` and commands.**

`DesktopState` contains `Mutex<Option<String>>` for raw input, `Mutex<Option<IssueContext>>` for the current context, and `RecentProjectStore`. `analyze_input` calls `Pipeline::default().analyze`, stores raw input and context in memory, then returns `{ context, candidates }`. `current_context` reruns the stored input at the selected budget; it returns `Ok(None)` when no analysis exists. `confirm_project` canonicalizes the selected path, rejects non-directories, and delegates to `RecentProjectStore`.

- [ ] **Step 3: Verify and commit.**

Run `cargo test -p devtriage-desktop commands::tests` and `cargo clippy -p devtriage-desktop --all-targets -- -D warnings`, then:

```bash
git add crates/devtriage-desktop/src
git commit -m "feat(desktop): add local analysis commands"
```

### Task 4: Add menu-bar and singleton detail-window lifecycle

**Files:** Modify `src/lib.rs`; create `tests/window_lifecycle.rs`.

**Interfaces:** `run()` registers commands, clipboard/dialog plugins, and `show_detail_window(app: &AppHandle) -> tauri::Result<()>`.

- [ ] **Step 1: Write lifecycle tests around a pure helper.**

```rust
#[test]
fn detail_label_is_stable() { assert_eq!(detail_window_label(), "detail"); }
#[test]
fn tray_click_requests_main_window() { assert_eq!(tray_click_action(MouseButton::Left), TrayAction::ShowMain); }
```

Run `cargo test -p devtriage-desktop window_lifecycle`. Expected: fail for absent helpers.

- [ ] **Step 2: Implement the lifecycle.**

Create a `TrayIconBuilder` with the default icon and `show_menu_on_left_click(false)`. On left mouse-up, retrieve `main`, unminimize, show, and focus it. Register a `show_detail_window` command: if label `detail` exists, show and focus it; otherwise create a `WebviewWindowBuilder` loading `index.html?view=detail`, sized `760x700`. Keep a Quit menu item only on right click.

- [ ] **Step 3: Verify manually and commit.**

Run `cargo test -p devtriage-desktop window_lifecycle`, then `pnpm --dir crates/devtriage-desktop/frontend dev` and `pnpm --dir crates/devtriage-desktop/frontend tauri dev`; verify tray click, Show Detail, and Quit manually. Commit:

```bash
git add crates/devtriage-desktop/src crates/devtriage-desktop/tests
git commit -m "feat(desktop): add tray and detail window lifecycle"
```

### Task 5: Build and test the quick-panel React workflow

**Files:** Create `frontend/src/{api,App,main,styles}.tsx`; create `frontend/src/App.test.tsx` and Vite/Vitest configuration.

**Interfaces:** `analyzeInput(input, budget)`, `confirmProject(path)`, `copyText(text)`, and `showDetail()` wrap the Tauri APIs; `App` owns input, response, error, and selected-candidate state.

- [ ] **Step 1: Write failing UI tests.**

```tsx
it("shows redacted summary and copies only on click", async () => {
  render(<App api={fakeApi({ text: "## Facts\n[REDACTED:CREDENTIAL]" })} />);
  await userEvent.type(screen.getByLabelText("Log input"), "fatal token=secret");
  await userEvent.click(screen.getByRole("button", { name: "Analyze" }));
  expect(await screen.findByText("[REDACTED:CREDENTIAL]")).toBeVisible();
  expect(fakeApi.copyText).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", { name: "Copy AI context" }));
  expect(fakeApi.copyText).toHaveBeenCalledWith(expect.not.stringContaining("secret"));
});
```

Run `pnpm --dir crates/devtriage-desktop/frontend test`. Expected: fail because `App` does not exist.

- [ ] **Step 2: Implement the UI and typed invoke boundary.**

`App` renders a labelled textarea, Analyze button, first error/analysis-depth summary, candidate radio list with reasons, Confirm project and Choose folder buttons, Copy AI context and Open detail buttons. Disable copy/detail until an analysis response exists. Surface command failures in `<p role="alert">`; do not render raw input in error details. `Choose folder` calls dialog `open({ directory: true, multiple: false })` and sends a non-null selected path to `confirmProject`.

- [ ] **Step 3: Verify and commit.**

Run `pnpm --dir crates/devtriage-desktop/frontend test` and `pnpm --dir crates/devtriage-desktop/frontend build`, then:

```bash
git add crates/devtriage-desktop/frontend
git commit -m "feat(desktop): add quick analysis panel"
```

### Task 6: Build the detail view, document use, and validate the release slice

**Files:** Create `frontend/src/Detail.tsx`, `frontend/src/Detail.test.tsx`; modify `frontend/src/main.tsx`, `README.md`.

**Interfaces:** `Detail` invokes `currentContext(budget)` on mount and re-invokes it after a `compact|standard|detailed` selection changes.

- [ ] **Step 1: Write the failing detail test.**

```tsx
it("loads the same context and switches output budgets", async () => {
  render(<Detail api={fakeApi()} />);
  expect(await screen.findByText("Evidence")).toBeVisible();
  await userEvent.selectOptions(screen.getByLabelText("Output budget"), "detailed");
  expect(fakeApi.currentContext).toHaveBeenLastCalledWith("detailed");
});
```

Run `pnpm --dir crates/devtriage-desktop/frontend test`. Expected: fail for missing `Detail`.

- [ ] **Step 2: Implement detail rendering and documentation.**

Use `new URLSearchParams(window.location.search).get("view") === "detail"` to render `Detail`, otherwise `App`. `Detail` displays evidence kind/value/provenance, transformations, fingerprint, output preview, `omitted_evidence`, and a budget select; it shows an empty state when `currentContext` returns null. Add README sections for `pnpm --dir crates/devtriage-desktop/frontend install`, `pnpm --dir crates/devtriage-desktop/frontend tauri dev`, the Node 20 prerequisite, and the explicit no-project-read/no-history/no-network boundary.

- [ ] **Step 3: Run the complete verification suite.**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir crates/devtriage-desktop/frontend test
pnpm --dir crates/devtriage-desktop/frontend build
```

Expected: all commands exit 0. Then manually paste `fatal token=hidden-value`, verify copy output lacks `hidden-value`, choose and confirm a directory, restart the app, and verify it appears as a recent candidate.

- [ ] **Step 4: Commit.**

```bash
git add README.md Cargo.lock crates/devtriage-desktop
git commit -m "docs: document desktop shell workflow"
```

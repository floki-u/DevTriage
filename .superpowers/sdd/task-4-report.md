# Task 4 report

## Change

Added `@tauri-apps/cli` `2.8.0` to `crates/devtriage-desktop/frontend` development dependencies and regenerated `pnpm-lock.yaml`. The exact 2.8.0 pin is compatible with the desktop crate's pinned Tauri `=2.8.3` / frontend Tauri 2.8 dependency line.

## Commands and results

```sh
source /Users/floki/.nvm/nvm.sh && nvm use 24 >/dev/null && pnpm --dir crates/devtriage-desktop/frontend add -D @tauri-apps/cli@2.8.x
```

Passed. pnpm installed `@tauri-apps/cli 2.8.0` and updated the lockfile.

```sh
cargo test -p devtriage-desktop --test window_lifecycle
```

Passed: 2 passed (`tray_click_requests_main_window`, `detail_label_is_stable`).

```sh
source /Users/floki/.nvm/nvm.sh && nvm use 24 >/dev/null && pnpm --dir=crates/devtriage-desktop/frontend test && pnpm --dir=crates/devtriage-desktop/frontend build
```

Passed: Vitest 1/1; TypeScript/Vite production build completed.

## Bounded desktop startup and manual verification

The requested `pnpm --dir crates/devtriage-desktop/frontend tauri dev` form was attempted under Node 24. With pnpm 11 it parsed the directory argument as a command; the equivalent `--dir=...` form found the newly installed CLI but exited before launching because the frontend directory has no `tauri.conf.json` (the config is at `crates/devtriage-desktop/tauri.conf.json`).

Starting the installed local CLI from the config directory reached Tauri's `BeforeDevCommand`:

```text
Running BeforeDevCommand (`pnpm --dir frontend dev --host 127.0.0.1`)
ENOENT: no such file or directory, lstat '.../crates/devtriage-desktop/frontend/frontend'
```

This is an existing relative-command/config working-directory issue, outside the dependency-only P1 scope. No desktop window or tray menu was created, so tray left-click, **Show Detail**, and **Quit** cannot be manually exercised in this session. The focused lifecycle test provides automated evidence for tray-click routing and the detail-window label only; it does not cover a live tray interaction or quit action.

## Startup configuration repair

Updated `crates/devtriage-desktop/tauri.conf.json` so `beforeDevCommand` is `pnpm dev --host 127.0.0.1` and `beforeBuildCommand` is `pnpm build`. Tauri executes both commands with the frontend directory as the working directory, so the previous `--dir frontend` form incorrectly resolved `frontend/frontend`. Restored the unrelated `tauri-build = { version = "2.3", features = [] }` manifest edit to its original `tauri-build = "2.3"` form.

With Node 24, a bounded local-CLI smoke test from `crates/devtriage-desktop` completed the repaired startup path using the installed frontend CLI binary:

```sh
./frontend/node_modules/.bin/tauri dev
```

Tauri ran `pnpm dev --host 127.0.0.1`, Vite served `http://127.0.0.1:5173/`, Cargo compiled `devtriage-desktop`, and the native `target/debug/devtriage-desktop` process launched. The test terminated its process group after 45 seconds. This confirms desktop and tray initialization reached native process startup, but does not provide interactive visual confirmation of the tray menu.

The requested pnpm shorthand was also attempted:

```sh
pnpm --dir=frontend tauri dev
```

Under pnpm 11 it changes the CLI working directory to `frontend`; the Tauri CLI only searches that directory and its descendants for `tauri.conf.json`, so it aborts before configuration loading because the config intentionally lives in the desktop crate parent. Use the local CLI binary above (or run the equivalent package command without pnpm's `--dir` cwd rewrite) from the desktop crate.

Post-repair checks passed:

```sh
cargo test -p devtriage-desktop --test window_lifecycle
pnpm --dir=crates/devtriage-desktop/frontend test
pnpm --dir=crates/devtriage-desktop/frontend build
```

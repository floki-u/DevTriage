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

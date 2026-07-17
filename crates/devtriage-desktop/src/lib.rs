pub mod commands;
pub mod domain;
pub mod projects;
pub mod recent;

use devtriage_core::IssueContext;
use std::path::Path;
use std::sync::Mutex;
use tauri::{
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub use domain::{CandidateReason, ProjectCandidate, RecentProject};
pub use projects::rank_projects;
pub use recent::{RecentProjectError, RecentProjectStore};

/// Application-local state. Raw logs and their analysis are deliberately never persisted.
pub struct DesktopState {
    raw_input: Mutex<Option<String>>,
    current_context: Mutex<Option<IssueContext>>,
    recent_projects: RecentProjectStore,
}

impl DesktopState {
    pub fn new(recent_projects: RecentProjectStore) -> Self {
        Self {
            raw_input: Mutex::new(None),
            current_context: Mutex::new(None),
            recent_projects,
        }
    }

    pub fn with_recent_projects_at(directory: impl AsRef<Path>) -> Self {
        Self::new(RecentProjectStore::at(directory))
    }

    #[cfg(test)]
    pub fn for_test() -> Self {
        Self::with_recent_projects_at(
            std::env::temp_dir().join(format!("devtriage-desktop-test-{}", std::process::id())),
        )
    }
}

const MAIN_WINDOW_LABEL: &str = "main";
const QUIT_MENU_ID: &str = "quit";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    ShowMain,
    Ignore,
}

pub fn detail_window_label() -> &'static str {
    "detail"
}

pub fn tray_click_action(button: MouseButton) -> TrayAction {
    match button {
        MouseButton::Left => TrayAction::ShowMain,
        MouseButton::Right | MouseButton::Middle => TrayAction::Ignore,
    }
}

pub fn should_show_main_on_reopen(has_visible_windows: bool) -> bool {
    !has_visible_windows
}

fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    let Some(main) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    main.unminimize()?;
    main.show()?;
    main.set_focus()?;
    Ok(())
}

pub fn show_detail_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(detail) = app.get_webview_window(detail_window_label()) {
        detail.show()?;
        detail.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app,
        detail_window_label(),
        WebviewUrl::App("index.html?view=detail".into()),
    )
    .inner_size(760.0, 700.0)
    .build()?;
    Ok(())
}

#[tauri::command(rename = "show_detail_window")]
fn show_detail_window_command(app: AppHandle) -> tauri::Result<()> {
    show_detail_window(&app)
}

fn configure_tray(app: &tauri::App) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;
    let icon = app
        .default_window_icon()
        .expect("the generated Tauri context must provide a default icon")
        .clone();

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id().as_ref() == QUIT_MENU_ID {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state: MouseButtonState::Up,
                ..
            } = event
                && tray_click_action(button) == TrayAction::ShowMain
            {
                let _ = show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            app.manage(DesktopState::new(RecentProjectStore::at(app_data)));
            configure_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analyze_input,
            commands::project_candidates,
            commands::confirm_project,
            commands::current_context,
            show_detail_window_command,
        ])
        .build(tauri::generate_context!())
        .expect("error while building DevTriage");

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } = event
            && should_show_main_on_reopen(has_visible_windows)
        {
            if let Err(error) = show_main_window(app) {
                eprintln!("app: failed to restore the main window: {error}");
            }
        }
    });
}

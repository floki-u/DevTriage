use devtriage_desktop::{TrayAction, detail_window_label, tray_click_action};
use tauri::tray::MouseButton;

#[test]
fn detail_label_is_stable() {
    assert_eq!(detail_window_label(), "detail");
}

#[test]
fn tray_click_requests_main_window() {
    assert_eq!(tray_click_action(MouseButton::Left), TrayAction::ShowMain);
}

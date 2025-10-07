mod bbs;
mod menu_bar;
mod popover;
mod system_tray;

use menu_bar::create_menu_bar;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use system_tray::SystemTray;

use crate::popover::PopoverViewController;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mtm = MainThreadMarker::new().unwrap();

    // NSApplicationの初期化
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let view_controller = PopoverViewController::new(mtm);

    // ショートカットキーに必要なメニューバーを設定
    let menu_bar = create_menu_bar(mtm, &view_controller, sel!(postButtonDidClick:));
    app.setMainMenu(Some(&menu_bar));

    // システムトレイを作成
    let _system_tray = SystemTray::new(mtm, view_controller);

    // イベントループを開始
    app.run();
}

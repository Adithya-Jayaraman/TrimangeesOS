mod application_manager;
mod boot;
mod config;
mod desktop;
mod desktop_state;
mod display;
mod shell;
mod start_menu;
mod taskbar;
mod window_manager;
mod desktop_icon;
mod webview_manager;

use boot::BootManager;

fn main() {
    println!("Powering on Trimangees OS...\n");
    let mut boot = BootManager::new();
    boot.boot();
    println!("\nOpening Display Server...");
    display::DisplayServer::run();
}
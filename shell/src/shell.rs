use crate::application_manager::{Application, ApplicationManager};
use crate::desktop::Desktop;
use crate::start_menu::StartMenu;
use crate::taskbar::Taskbar;
use crate::window_manager::WindowManager;

pub struct Shell {
    pub window_manager: WindowManager,
    pub desktop: Desktop,
    pub taskbar: Taskbar,
    pub start_menu: StartMenu,
    pub application_manager: ApplicationManager,
}

impl Shell {
    pub fn new() -> Self {
        let mut application_manager = ApplicationManager::new();

        application_manager.register_app(
            Application::new(
                "Browser",
                "browser",
                "1.0",
                "Internet",
            ),
        );

        application_manager.register_app(
            Application::new(
                "Explorer",
                "explorer",
                "1.0",
                "System",
            ),
        );

        application_manager.register_app(
            Application::new(
                "Terminal",
                "terminal",
                "1.0",
                "System",
            ),
        );

        application_manager.register_app(
            Application::new(
                "Docs",
                "docs",
                "1.0",
                "Productivity",
            ),
        );

        Self {
            window_manager: WindowManager::new(),
            desktop: Desktop::new(),
            taskbar: Taskbar::new(),
            start_menu: StartMenu::new(),
            application_manager,
        }
    }

    pub fn initialize(&self) {
        println!("Initializing Desktop Environment...");
        println!("Desktop Loaded.");
        println!("Window Manager Loaded.");
        println!("Taskbar Loaded.");
        println!("Start Menu Loaded.");
    }

    pub fn launch_app(
        &mut self,
        app_name: &str,
    ) {
        if let Some(app) = self.application_manager.find_app(app_name) {

            println!("Launching {}...", app.name);

self.window_manager.create_window(
    app.name.clone(),
    app.executable.clone(),
    900,
    700,
);

            self.taskbar.add_running_app(&app.name);

        } else {

            println!(
                "Application '{}' not found.",
                app_name
            );

        }
    }

    pub fn toggle_start_menu(
        &mut self,
    ) {
        self.start_menu.toggle();
    }
}
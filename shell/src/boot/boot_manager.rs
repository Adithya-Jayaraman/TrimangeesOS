use crate::config::ConfigManager;
use crate::shell::Shell;

pub struct BootManager {
    config_manager: ConfigManager,
    shell: Shell,
}

impl BootManager {
    pub fn new() -> Self {
        println!("Creating Configuration Manager...");
        let config_manager = ConfigManager::new();

        println!("Creating Shell...");
        let shell = Shell::new();

        Self {
            config_manager,
            shell,
        }
    }

    pub fn boot(&mut self) {
        println!();
        println!("==============================");
        println!("     Trimangees OS Boot");
        println!("==============================");
        println!();

        println!("[1/5] Loading Configuration...");
        self.config_manager.print_settings();

        println!();

        println!("[2/5] Initializing Desktop...");
        self.shell.initialize();

        println!();

        println!("[3/5] Registered Applications:");
        self.shell.application_manager.list_apps();

        println!();

        println!("[4/5] Testing Application Launch...");
        self.shell.launch_app("Browser");

        println!();

        println!("[5/5] Boot Complete!");
        println!();

        println!("Trimangees OS successfully started.");
    }
}
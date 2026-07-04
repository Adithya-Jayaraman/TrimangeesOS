use super::app::Application;
use crate::window_manager::WindowManager;

pub struct ApplicationManager {
    applications: Vec<Application>,
}

impl ApplicationManager {

    pub fn new() -> Self {
        Self { applications: Vec::new() }
    }

    pub fn register_app(&mut self, app: Application) {
        self.applications.push(app);
    }

    pub fn list_apps(&self) {
        println!("Installed Applications:");
        for app in &self.applications {
            println!("{} ({})", app.name, app.version);
        }
    }

    pub fn find_app(&self, name: &str) -> Option<&Application> {
        self.applications.iter().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    pub fn launch(&self, executable: &str, window_manager: &mut WindowManager) {
        match executable {
            "explorer" => {
                window_manager.create_window("File Explorer".into(), "explorer".into(), 820, 540);
            }
            "browser" => {
                // Locate the browser directory:
                // 1. TRIMANGEES_BROWSER_DIR env var (set manually or by installer)
                // 2. A "browser" folder next to the shell binary (for ISO/release)
                // 3. Hardcoded sibling folder during development
                let browser_dir = std::env::var("TRIMANGEES_BROWSER_DIR")
                    .unwrap_or_else(|_| {
                        let exe = std::env::current_exe().unwrap_or_default();
                        let next_to_binary = exe.parent()
                            .map(|p| p.join("browser"))
                            .filter(|p| p.exists());
                        if let Some(p) = next_to_binary {
                            p.to_string_lossy().to_string()
                        } else {
                            // Dev fallback — browser folder alongside shell/
                            let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
                            manifest.parent()
                                .map(|p| p.join("browser").to_string_lossy().to_string())
                                .unwrap_or_else(|| "../browser".to_string())
                        }
                    });

                // On Windows: node_modules/.bin/electron.cmd
                // On Linux:   node_modules/.bin/electron
                let electron_cmd = {
                    let local_win  = format!("{}\\node_modules\\.bin\\electron.cmd", browser_dir);
                    let local_unix = format!("{}/node_modules/.bin/electron", browser_dir);
                    if std::path::Path::new(&local_win).exists() {
                        local_win
                    } else if std::path::Path::new(&local_unix).exists() {
                        local_unix
                    } else {
                        // Last resort: hope electron is in PATH
                        "electron".to_string()
                    }
                };

                let launched = std::process::Command::new(&electron_cmd)
                    .arg(".")
                    .current_dir(&browser_dir)
                    .spawn()
                    .is_ok();

                if !launched {
                    // Show placeholder so at least something opens
                    window_manager.create_window(
                        "Trimangees Browser".into(), "browser".into(), 1100, 700);
                }
            }
            "terminal" => {
                window_manager.create_window("Terminal".into(), "terminal".into(), 720, 420);
            }
            "tridocs" => {
                // Try to open local file first, then fall back to online
                let assets_dir = std::env::var("TRIMANGEES_ASSETS_DIR")
                    .unwrap_or_else(|_| "./apps".to_string());
                let local = format!("{}/tridocs.html", assets_dir);
                let launched = if std::path::Path::new(&local).exists() {
                    std::process::Command::new("xdg-open").arg(&local).spawn().is_ok()
                } else {
                    std::process::Command::new("xdg-open")
                        .arg("https://trimangees.netlify.app")
                        .spawn().is_ok()
                };
                if !launched {
                    window_manager.create_window("TRiDOCS".into(), "tridocs".into(), 900, 600);
                }
            }
            "trisheets" => {
                let launched = std::process::Command::new("xdg-open")
                    .arg("https://trimangees.netlify.app/trisheets")
                    .spawn().is_ok();
                if !launched {
                    window_manager.create_window("TRiSHEETS".into(), "trisheets".into(), 900, 600);
                }
            }
            "trislides" => {
                let launched = std::process::Command::new("xdg-open")
                    .arg("https://trimangees.netlify.app/trislides")
                    .spawn().is_ok();
                if !launched {
                    window_manager.create_window("TRiSLIDES".into(), "trislides".into(), 900, 600);
                }
            }
            "tridraw" => {
                let launched = std::process::Command::new("xdg-open")
                    .arg("https://trimangees.netlify.app/tridraw")
                    .spawn().is_ok();
                if !launched {
                    window_manager.create_window("TRiDRAW".into(), "tridraw".into(), 900, 600);
                }
            }
            "settings" => {
                window_manager.create_window("Settings".into(), "settings".into(), 680, 480);
            }
            _ => {
                // Try launching as a system executable
                let _ = std::process::Command::new(executable).spawn();
            }
        }
    }
}
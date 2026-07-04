pub struct Taskbar {
    pub running_apps: Vec<String>,
    pub start_button_visible: bool,
}

impl Taskbar {

    pub fn new() -> Self {
        Self {
            running_apps: Vec::new(),
            start_button_visible: true,
        }
    }

    pub fn add_running_app(&mut self, app_name: &str) {
        self.running_apps.push(app_name.to_string());
    }

    pub fn remove_running_app(&mut self, app_name: &str) {
        self.running_apps.retain(|app| app != app_name);
    }

    pub fn list_running_apps(&self) {
        println!("Running Applications:");

        for app in &self.running_apps {
            println!("{}", app);
        }
    }
}
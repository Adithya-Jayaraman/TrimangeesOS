pub struct StartMenu {
    pub visible: bool,
    pub apps: Vec<String>,
}

impl StartMenu {

    pub fn new() -> Self {
        Self {
            visible: false,
            apps: Vec::new(),
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        println!("Start Menu opened.");
    }

    pub fn close(&mut self) {
        self.visible = false;
        println!("Start Menu closed.");
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;

        if self.visible {
            println!("Start Menu opened.");
        } else {
            println!("Start Menu closed.");
        }
    }

    pub fn add_app(&mut self, app_name: &str) {
        self.apps.push(app_name.to_string());
    }

    pub fn list_apps(&self) {
        println!("Applications:");

        for app in &self.apps {
            println!("{}", app);
        }
    }
}
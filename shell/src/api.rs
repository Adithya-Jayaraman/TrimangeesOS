use crate::shell::Shell;

pub struct ShellApi {
    shell: Shell,
}

impl ShellApi {

    pub fn new() -> Self {
        Self {
            shell: Shell::new(),
        }
    }

    pub fn launch_app(
        &mut self,
        app_name: &str,
    ) {
        self.shell.launch_app(app_name);
    }

    pub fn open_start_menu(
        &mut self,
    ) {
        self.shell.toggle_start_menu();
    }
}
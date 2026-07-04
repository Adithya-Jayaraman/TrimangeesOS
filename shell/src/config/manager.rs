use super::config::Config;

pub struct ConfigManager {

    config: Config,

}

impl ConfigManager {

    pub fn new() -> Self {

        Self {

            config: Config::default(),

        }

    }

    pub fn get_config(&self) -> &Config {

        &self.config

    }

    pub fn set_wallpaper(
        &mut self,
        wallpaper: &str,
    ) {

        self.config.wallpaper = wallpaper.to_string();

    }

    pub fn set_theme(
        &mut self,
        theme: &str,
    ) {

        self.config.theme = theme.to_string();

    }

    pub fn print_settings(&self) {

        println!("Current Settings:");

        println!("Wallpaper: {}", self.config.wallpaper);

        println!("Theme: {}", self.config.theme);

        println!("Accent: {}", self.config.accent_color);

        println!("Browser: {}", self.config.default_browser);

        println!("Terminal: {}", self.config.default_terminal);

    }

}
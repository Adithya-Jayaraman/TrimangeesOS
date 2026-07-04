#[derive(Clone, Debug)]
pub struct Config {

    pub wallpaper: String,
    pub theme: String,
    pub accent_color: String,
    pub default_browser: String,
    pub default_terminal: String,

    // Accent RGB for renderer
    pub accent_r: u8,
    pub accent_g: u8,
    pub accent_b: u8,
    pub wallpaper_preset: u8,
}

impl Config {

    pub fn default() -> Self {
        Self {
            wallpaper:        String::from("galaxy.jpg"),
            theme:            String::from("dark"),
            accent_color:     String::from("purple"),
            default_browser:  String::from("Browser"),
            default_terminal: String::from("Terminal"),
            accent_r:         72,
            accent_g:         138,
            accent_b:         240,
            wallpaper_preset: 0,
        }
    }

    fn config_path() -> std::path::PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home)
            .join(".config")
            .join("trimangees")
            .join("shell.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let mut cfg = Config::default();
        let Ok(text) = std::fs::read_to_string(&path) else { return cfg; };

        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') || !line.contains('=') { continue; }
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim().trim_matches('"');
            match key {
                "theme"            => cfg.theme = val.to_string(),
                "wallpaper"        => cfg.wallpaper = val.to_string(),
                "wallpaper_preset" => cfg.wallpaper_preset = val.parse().unwrap_or(0),
                "accent_color"     => cfg.accent_color = val.to_string(),
                "accent_r"         => cfg.accent_r  = val.parse().unwrap_or(72),
                "accent_g"         => cfg.accent_g  = val.parse().unwrap_or(138),
                "accent_b"         => cfg.accent_b  = val.parse().unwrap_or(240),
                _ => {}
            }
        }
        cfg
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let text = format!(
            "# Trimangees OS — shell config\ntheme = \"{}\"\nwallpaper = \"{}\"\nwallpaper_preset = {}\naccent_color = \"{}\"\naccent_r = {}\naccent_g = {}\naccent_b = {}\n",
            self.theme, self.wallpaper, self.wallpaper_preset,
            self.accent_color, self.accent_r, self.accent_g, self.accent_b
        );
        let _ = std::fs::write(&path, text);
    }
}
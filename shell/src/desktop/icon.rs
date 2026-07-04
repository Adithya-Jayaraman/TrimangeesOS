pub struct DesktopIcon {
    pub name: String,
    pub image: String,
    pub app_name: String,
    pub x: i32,
    pub y: i32,
}

impl DesktopIcon {
    pub fn new(
        name: &str,
        image: &str,
        app_name: &str,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            app_name: app_name.to_string(),
            x,
            y,
        }
    }
}
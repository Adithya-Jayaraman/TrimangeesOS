use std::time::Instant;

pub struct DesktopIcon {
    pub x: i32,
    pub y: i32,
    pub selected: bool,
    pub name: String,
    pub app_name: String,

    pub last_click: Option<Instant>,
}

impl DesktopIcon {
    pub fn new(
    name: &str,
    app_name: &str,
    x: i32,
    y: i32,
) -> Self {
        Self {
    name: name.to_string(),
    app_name: app_name.to_string(),
    x,
    y,
    selected: false,
    last_click: None,
}
    }

    pub fn contains(&self, mx: i32, my: i32) -> bool {
        mx >= self.x
            && mx <= self.x + 48
            && my >= self.y
            && my <= self.y + 48
    }
}
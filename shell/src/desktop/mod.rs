mod grid;
mod icon;

pub use grid::DesktopGrid;
pub use icon::DesktopIcon;

pub struct Desktop {
    pub wallpaper: String,
    pub icons: Vec<DesktopIcon>,
    pub grid: DesktopGrid,
}

impl Desktop {
    pub fn new() -> Self {
        Self {
            wallpaper: String::from("default_wallpaper.png"),
            icons: Vec::new(),
            grid: DesktopGrid::new(),
        }
    }

    pub fn add_icon(
        &mut self,
        name: &str,
        image: &str,
        app_name: &str,
    ) {
        let (x, y) = self.grid.get_position(self.icons.len());

        let icon = DesktopIcon::new(
            name,
            image,
            app_name,
            x,
            y,
        );

        self.icons.push(icon);
    }

    pub fn list_icons(&self) {
        println!("Desktop Icons:");

        for icon in &self.icons {
            println!(
                "{} at ({}, {})",
                icon.name,
                icon.x,
                icon.y
            );
        }
    }
}
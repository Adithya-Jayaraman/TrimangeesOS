pub struct DesktopGrid {
    pub start_x: i32,
    pub start_y: i32,
    pub cell_width: i32,
    pub cell_height: i32,
    pub columns: i32,
}

impl DesktopGrid {
    pub fn new() -> Self {
        Self {
            start_x: 40,
            start_y: 40,
            cell_width: 120,
            cell_height: 120,
            columns: 8,
        }
    }

    pub fn get_position(&self, index: usize) -> (i32, i32) {
        let column = (index as i32) % self.columns;
        let row = (index as i32) / self.columns;

        let x = self.start_x + column * self.cell_width;
        let y = self.start_y + row * self.cell_height;

        (x, y)
    }
}
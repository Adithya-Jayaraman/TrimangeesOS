pub struct Cursor {
    pub x: i32,
    pub y: i32,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 100,
            y: 100,
        }
    }
}
// ============================================================
// StartMenu — Windows 11 style: centred, wide, search at top
// ============================================================

#[derive(Clone)]
pub struct StartMenuEntry {
    pub name:       String,
    pub executable: String,
    pub icon_color: [u8; 3],
    pub icon_char:  char,   // single letter shown inside icon square
}

pub struct StartMenu {
    pub visible:       bool,
    pub entries:       Vec<StartMenuEntry>,
    pub hovered:       Option<usize>,
    pub power_hovered: Option<usize>,
    pub search_query:  String,   // future: filter entries
}

// Layout — centred on screen, tall and wide like Windows 11
pub const MENU_W:          usize = 520;
pub const MENU_H:          usize = 420;
// Position: centred horizontally, sits just above taskbar
pub const MENU_X:          usize = (1280 - MENU_W) / 2;   // = 380
pub const MENU_Y_FROM_BOTTOM: usize = MENU_H + 42;        // 42 = taskbar height + gap

// Tile grid
pub const TILE_COLS:       usize = 4;
pub const TILE_W:          usize = 96;
pub const TILE_H:          usize = 80;
pub const TILE_GAP:        usize = 10;
pub const TILE_PAD_X:      usize = (MENU_W - TILE_COLS * TILE_W - (TILE_COLS-1)*TILE_GAP) / 2;
pub const TILE_PAD_Y:      usize = 90; // below search bar + "Pinned" label

impl StartMenu {
    pub fn new() -> Self {
        Self {
            visible:       false,
            hovered:       None,
            power_hovered: None,
            search_query:  String::new(),
            entries: vec![
                StartMenuEntry { name:"File Explorer".into(), executable:"explorer".into(),  icon_color:[245,180,35],  icon_char:'F' },
                StartMenuEntry { name:"Terminal".into(),      executable:"terminal".into(),  icon_color:[60,200,80],   icon_char:'>' },
                StartMenuEntry { name:"Browser".into(),       executable:"browser".into(),   icon_color:[60,150,255],  icon_char:'B' },
                StartMenuEntry { name:"TRiDOCS".into(),       executable:"tridocs".into(),   icon_color:[70,130,220],  icon_char:'D' },
                StartMenuEntry { name:"TRiSHEETS".into(),     executable:"trisheets".into(), icon_color:[50,180,100],  icon_char:'S' },
                StartMenuEntry { name:"TRiSLIDES".into(),     executable:"trislides".into(), icon_color:[220,100,60],  icon_char:'P' },
                StartMenuEntry { name:"TRiDRAW".into(),       executable:"tridraw".into(),   icon_color:[180,80,220],  icon_char:'A' },
                StartMenuEntry { name:"Settings".into(),      executable:"settings".into(),  icon_color:[130,135,170], icon_char:'⚙' },
            ],
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.hovered       = None;
            self.power_hovered = None;
            self.search_query.clear();
        }
    }

    pub fn tile_at(&self, mx: i32, my: i32, screen_height: usize) -> Option<usize> {
        if !self.visible { return None; }
        let menu_top = (screen_height - MENU_Y_FROM_BOTTOM) as i32;
        for (i, _) in self.entries.iter().enumerate() {
            let (tx, ty) = Self::tile_pos(i, menu_top);
            if mx >= tx && mx < tx + TILE_W as i32
            && my >= ty && my < ty + TILE_H as i32 {
                return Some(i);
            }
        }
        None
    }

    pub fn tile_pos(i: usize, menu_top: i32) -> (i32, i32) {
        let col = i % TILE_COLS;
        let row = i / TILE_COLS;
        let x = MENU_X as i32 + TILE_PAD_X as i32
              + col as i32 * (TILE_W as i32 + TILE_GAP as i32);
        let y = menu_top + TILE_PAD_Y as i32
              + row as i32 * (TILE_H as i32 + TILE_GAP as i32);
        (x, y)
    }

    pub fn power_btn_at(&self, mx: i32, my: i32, screen_height: usize) -> Option<usize> {
        if !self.visible { return None; }
        let menu_top = (screen_height - MENU_Y_FROM_BOTTOM) as i32;
        let btn_y = menu_top + MENU_H as i32 - 42;
        let btn_h = 28i32;
        if my < btn_y || my > btn_y + btn_h { return None; }

        let btn_w = 88i32; let gap = 10i32;
        let total = 3 * btn_w + 2 * gap;
        let start_x = MENU_X as i32 + (MENU_W as i32 - total) / 2;
        for i in 0..3usize {
            let bx = start_x + i as i32 * (btn_w + gap);
            if mx >= bx && mx < bx + btn_w { return Some(i); }
        }
        None
    }
}
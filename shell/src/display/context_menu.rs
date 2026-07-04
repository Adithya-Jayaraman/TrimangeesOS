// ============================================================
// ContextMenu — right-click popup on desktop or window
// ============================================================

#[derive(Clone, PartialEq)]
pub enum ContextTarget {
    Desktop,
    Window(u32),
}

#[derive(Clone)]
pub struct ContextMenuItem {
    pub label:     String,
    pub action:    String,   // identifier handled in display_server
    pub separator: bool,     // if true, draw a divider line instead
}

pub struct ContextMenu {
    pub visible:  bool,
    pub x:        i32,
    pub y:        i32,       // screen position of top-left
    pub target:   ContextTarget,
    pub items:    Vec<ContextMenuItem>,
    pub hovered:  Option<usize>,
}

pub const MENU_ITEM_H:  usize = 26;
pub const MENU_ITEM_W:  usize = 200;
pub const MENU_PAD_Y:   usize = 6;   // vertical padding top/bottom

impl ContextMenu {
    pub fn new() -> Self {
        Self {
            visible: false,
            x: 0,
            y: 0,
            target: ContextTarget::Desktop,
            items: Vec::new(),
            hovered: None,
        }
    }

    /// Open a desktop right-click menu at (mx, my)
    pub fn open_desktop(
        &mut self,
        mx: i32,
        my: i32,
        screen_w: usize,
        screen_h: usize,
    ) {
        self.items = vec![
            ContextMenuItem { label: "Open Terminal".into(),       action: "open_terminal".into(),  separator: false },
            ContextMenuItem { label: "Open File Explorer".into(),  action: "open_explorer".into(),  separator: false },
            ContextMenuItem { label: "".into(),                    action: "".into(),               separator: true  },
            ContextMenuItem { label: "New Folder".into(),          action: "new_folder".into(),     separator: false },
            ContextMenuItem { label: "".into(),                    action: "".into(),               separator: true  },
            ContextMenuItem { label: "Wallpaper Settings".into(),  action: "wallpaper".into(),      separator: false },
            ContextMenuItem { label: "About Trimangees OS".into(), action: "about".into(),          separator: false },
        ];
        self.target  = ContextTarget::Desktop;
        self.visible = true;
        self.hovered = None;
        self.place(mx, my, screen_w, screen_h);
    }

    /// Open a window chrome right-click menu
    pub fn open_window(&mut self, mx: i32, my: i32, win_id: u32, screen_w: usize, screen_h: usize) {
        self.items = vec![
            ContextMenuItem { label: "Restore".into(),   action: "restore".into(),   separator: false },
            ContextMenuItem { label: "Minimize".into(),  action: "minimize".into(),  separator: false },
            ContextMenuItem { label: "Maximize".into(),  action: "maximize".into(),  separator: false },
            ContextMenuItem { label: "".into(),          action: "".into(),          separator: true  },
            ContextMenuItem { label: "Close".into(),     action: "close".into(),     separator: false },
        ];
        self.target  = ContextTarget::Window(win_id);
        self.visible = true;
        self.hovered = None;
        self.place(mx, my, screen_w, screen_h);
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.hovered = None;
    }

    /// Total pixel height of the menu
    pub fn height(&self) -> usize {
        let mut h = MENU_PAD_Y * 2;
        for item in &self.items {
            h += if item.separator { 9 } else { MENU_ITEM_H };
        }
        h
    }

    /// Adjust position so menu stays on screen
    fn place(&mut self, mx: i32, my: i32, sw: usize, sh: usize) {
        let mw = MENU_ITEM_W as i32;
        let mh = self.height() as i32;
        self.x = if mx + mw > sw as i32 { sw as i32 - mw - 4 } else { mx };
        self.y = if my + mh > sh as i32 { sh as i32 - mh - 4 } else { my };
    }

    /// Return the item index the mouse is over, if any
    pub fn item_at(&self, mx: i32, my: i32) -> Option<usize> {
        if !self.visible { return None; }
        if mx < self.x || mx > self.x + MENU_ITEM_W as i32 { return None; }

        let mut cy = self.y + MENU_PAD_Y as i32;
        for (i, item) in self.items.iter().enumerate() {
            let h = if item.separator { 9i32 } else { MENU_ITEM_H as i32 };
            if !item.separator && my >= cy && my < cy + h {
                return Some(i);
            }
            cy += h;
        }
        None
    }

    /// Pixel y of item i's top edge (for hit testing / hover highlight)
    pub fn item_y(&self, idx: usize) -> i32 {
        let mut cy = self.y + MENU_PAD_Y as i32;
        for (i, item) in self.items.iter().enumerate() {
            if i == idx { return cy; }
            cy += if item.separator { 9 } else { MENU_ITEM_H as i32 };
        }
        cy
    }
}
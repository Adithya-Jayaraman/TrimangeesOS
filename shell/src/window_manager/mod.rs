// ============================================================
// Snap zones
// ============================================================
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SnapZone {
    Left, Right, TopLeft, TopRight, BottomLeft, BottomRight, Maximize,
}

const SCREEN_W: i32 = 1280;
const SCREEN_H: i32 = 720;
const TASKBAR:  i32 = 38;
const SNAP_MARGIN: i32 = 24;

impl SnapZone {
    pub fn detect(mx: i32, my: i32) -> Option<SnapZone> {
        let at_left   = mx < SNAP_MARGIN;
        let at_right  = mx > SCREEN_W - SNAP_MARGIN;
        let at_top    = my < SNAP_MARGIN;
        match (at_top, at_left, at_right) {
            (true,  _,     _    ) => Some(SnapZone::Maximize),
            (_,     true,  _    ) if my < SCREEN_H/2 => Some(SnapZone::TopLeft),
            (_,     true,  _    ) => Some(SnapZone::BottomLeft),
            (_,     _,     true ) if my < SCREEN_H/2 => Some(SnapZone::TopRight),
            (_,     _,     true ) => Some(SnapZone::BottomRight),
            _ => None,
        }
    }

    pub fn geometry(self) -> (i32, i32, u32, u32) {
        let hw = (SCREEN_W / 2) as u32;
        let hh = ((SCREEN_H - TASKBAR) / 2) as u32;
        let fh = (SCREEN_H - TASKBAR) as u32;
        match self {
            SnapZone::Left        => (0,           0,    hw, fh),
            SnapZone::Right       => (SCREEN_W/2,  0,    hw, fh),
            SnapZone::TopLeft     => (0,           0,    hw, hh),
            SnapZone::TopRight    => (SCREEN_W/2,  0,    hw, hh),
            SnapZone::BottomLeft  => (0,           SCREEN_H/2-TASKBAR/2, hw, hh),
            SnapZone::BottomRight => (SCREEN_W/2,  SCREEN_H/2-TASKBAR/2, hw, hh),
            SnapZone::Maximize    => (0,           0,    SCREEN_W as u32, fh),
        }
    }
}

// ============================================================
// Resize edges
// ============================================================
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResizeEdge {
    Top, Bottom, Left, Right, TopLeft, TopRight, BottomLeft, BottomRight,
}

pub const RESIZE_MARGIN: i32 = 8;

// ============================================================
// File Explorer state
// ============================================================
#[derive(Clone)]
pub struct FileEntry {
    pub name:     String,
    pub is_dir:   bool,
    pub size:     u64,
    pub ext:      String,
}

pub struct ExplorerState {
    pub current_path:   String,
    pub entries:        Vec<FileEntry>,
    pub selected:       Option<usize>,
    pub scroll_offset:  usize,
    pub history:        Vec<String>,
    pub history_idx:    usize,
}

impl ExplorerState {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/".to_string());
        let mut s = Self {
            current_path:  home.clone(),
            entries:       Vec::new(),
            selected:      None,
            scroll_offset: 0,
            history:       vec![home.clone()],
            history_idx:   0,
        };
        s.refresh();
        s
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        self.selected = None;
        let path = std::path::Path::new(&self.current_path);
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut dirs:  Vec<FileEntry> = Vec::new();
            let mut files: Vec<FileEntry> = Vec::new();
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; } // skip hidden
                let is_dir = e.path().is_dir();
                let size   = e.metadata().map(|m| m.len()).unwrap_or(0);
                let ext    = std::path::Path::new(&name)
                    .extension()
                    .map(|x| x.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let entry = FileEntry { name, is_dir, size, ext };
                if is_dir { dirs.push(entry); } else { files.push(entry); }
            }
            dirs.sort_by(|a,b| a.name.cmp(&b.name));
            files.sort_by(|a,b| a.name.cmp(&b.name));
            self.entries = dirs.into_iter().chain(files).collect();
        }
    }

    pub fn navigate(&mut self, path: String) {
        // Truncate forward history
        self.history.truncate(self.history_idx + 1);
        self.history.push(path.clone());
        self.history_idx = self.history.len() - 1;
        self.current_path = path;
        self.scroll_offset = 0;
        self.refresh();
    }

    pub fn go_back(&mut self) {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            self.current_path = self.history[self.history_idx].clone();
            self.scroll_offset = 0;
            self.refresh();
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = std::path::Path::new(&self.current_path).parent() {
            self.navigate(parent.to_string_lossy().to_string());
        }
    }

    pub fn go_forward(&mut self) {
        if self.history_idx + 1 < self.history.len() {
            self.history_idx += 1;
            self.current_path = self.history[self.history_idx].clone();
            self.scroll_offset = 0;
            self.refresh();
        }
    }
}

// ============================================================
// Settings state
// ============================================================
pub struct SettingsState {
    pub active_section: usize, // 0=Display 1=Personalise 2=About
    pub wallpaper_preset: u8,
    pub accent_r: u8,
    pub accent_g: u8,
    pub accent_b: u8,
    pub dark_mode: bool,
    pub hostname: String,
    pub os_version: String,
}

impl SettingsState {
    pub fn new() -> Self {
        let hostname = std::fs::read_to_string("/etc/hostname")
            .unwrap_or_else(|_| "trimangees-pc".to_string())
            .trim().to_string();
        Self {
            active_section:  0,
            wallpaper_preset: 0,
            accent_r: 72,
            accent_g: 138,
            accent_b: 240,
            dark_mode: true,
            hostname,
            os_version: "Trimangees OS 1.0".to_string(),
        }
    }
}

// ============================================================
// Window struct
// ============================================================
pub struct Window {
    pub id:     u32,
    pub title:  String,
    pub app:    String,
    pub x:      i32,
    pub y:      i32,
    pub width:  u32,
    pub height: u32,
    pub dragging:  bool,
    pub active:    bool,
    pub maximized: bool,
    pub minimized: bool,
    pub old_x:     i32,
    pub old_y:     i32,
    pub old_width:  u32,
    pub old_height: u32,
    pub min_width:  u32,
    pub min_height: u32,

    // Terminal state
    pub input_buffer:  String,
    pub output_lines:  Vec<String>,
    pub cursor_blink:  bool,
    pub scroll_offset: usize,
    pub history:       Vec<String>,
    pub history_index: Option<usize>,
    pub pending_output: Vec<String>,
    pub running:        bool,

    // Explorer state (only when app=="explorer")
    pub explorer: Option<ExplorerState>,

    // Settings state (only when app=="settings")
    pub settings: Option<SettingsState>,
}

pub struct WindowManager {
    pub windows: Vec<Window>,
    next_id: u32,
}

impl WindowManager {
    pub fn new() -> Self {
        Self { windows: Vec::new(), next_id: 1 }
    }

    pub fn resize_edge_at(window: &Window, mx: i32, my: i32) -> Option<ResizeEdge> {
        let x0 = window.x; let y0 = window.y;
        let x1 = window.x + window.width  as i32;
        let y1 = window.y + window.height as i32;
        let m  = RESIZE_MARGIN;
        if mx < x0-m || mx > x1+m || my < y0-m || my > y1+m { return None; }
        let on_l = mx >= x0-m && mx <= x0+m;
        let on_r = mx >= x1-m && mx <= x1+m;
        let on_t = my >= y0-m && my <= y0+m;
        let on_b = my >= y1-m && my <= y1+m;
        match (on_t, on_b, on_l, on_r) {
            (true,_,true,_)  => Some(ResizeEdge::TopLeft),
            (true,_,_,true)  => Some(ResizeEdge::TopRight),
            (_,true,true,_)  => Some(ResizeEdge::BottomLeft),
            (_,true,_,true)  => Some(ResizeEdge::BottomRight),
            (true,_,_,_)     => Some(ResizeEdge::Top),
            (_,true,_,_)     => Some(ResizeEdge::Bottom),
            (_,_,true,_)     => Some(ResizeEdge::Left),
            (_,_,_,true)     => Some(ResizeEdge::Right),
            _                => None,
        }
    }

    pub fn apply_resize(w: &mut Window, edge: ResizeEdge,
        smx:i32,smy:i32,sw:u32,sh:u32,swx:i32,swy:i32,mx:i32,my:i32)
    {
        let dx=mx-smx; let dy=my-smy;
        let mw=w.min_width as i32; let mh=w.min_height as i32;
        match edge {
            ResizeEdge::Right       => { w.width  = ((sw as i32+dx).max(mw)) as u32; }
            ResizeEdge::Bottom      => { w.height = ((sh as i32+dy).max(mh)) as u32; }
            ResizeEdge::Left        => { let nw=(sw as i32-dx).max(mw); w.x=swx+(sw as i32-nw); w.width=nw as u32; }
            ResizeEdge::Top         => { let nh=(sh as i32-dy).max(mh); w.y=swy+(sh as i32-nh); w.height=nh as u32; }
            ResizeEdge::BottomRight => { w.width=((sw as i32+dx).max(mw)) as u32; w.height=((sh as i32+dy).max(mh)) as u32; }
            ResizeEdge::BottomLeft  => { let nw=(sw as i32-dx).max(mw); w.x=swx+(sw as i32-nw); w.width=nw as u32; w.height=((sh as i32+dy).max(mh)) as u32; }
            ResizeEdge::TopRight    => { let nh=(sh as i32-dy).max(mh); w.y=swy+(sh as i32-nh); w.width=((sw as i32+dx).max(mw)) as u32; w.height=nh as u32; }
            ResizeEdge::TopLeft     => { let nw=(sw as i32-dx).max(mw); let nh=(sh as i32-dy).max(mh); w.x=swx+(sw as i32-nw); w.y=swy+(sh as i32-nh); w.width=nw as u32; w.height=nh as u32; }
        }
    }

    pub fn toggle_maximize(&mut self, id: u32) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id==id) {
            if !w.maximized {
                w.old_x=w.x; w.old_y=w.y; w.old_width=w.width; w.old_height=w.height;
                w.x=0; w.y=0; w.width=1280; w.height=682; w.maximized=true;
            } else {
                w.x=w.old_x; w.y=w.old_y; w.width=w.old_width; w.height=w.old_height; w.maximized=false;
            }
        }
    }

    pub fn minimize_window(&mut self, id: u32) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id==id) {
            w.minimized=true; w.active=false;
        }
        let top = self.windows.iter().rev().find(|w| !w.minimized).map(|w| w.id);
        for w in &mut self.windows { w.active = Some(w.id)==top; }
    }

    pub fn restore_window(&mut self, id: u32) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id==id) { w.minimized=false; }
        self.bring_to_front(id);
    }

    pub fn snap_window(&mut self, id: u32, zone: SnapZone) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id==id) {
            if !w.maximized { w.old_x=w.x; w.old_y=w.y; w.old_width=w.width; w.old_height=w.height; }
            let (x,y,width,height) = zone.geometry();
            w.x=x; w.y=y; w.width=width; w.height=height;
            if zone==SnapZone::Maximize { w.maximized=true; }
        }
    }

    pub fn window_at(&self, mx: i32, my: i32) -> Option<u32> {
        self.windows.iter().rev().find(|w| {
            mx>=w.x && mx<=w.x+w.width as i32 && my>=w.y && my<=w.y+32
        }).map(|w| w.id)
    }

    pub fn window_body_at(&self, mx: i32, my: i32) -> Option<u32> {
        self.windows.iter().rev().find(|w| {
            mx>=w.x && mx<=w.x+w.width as i32 && my>=w.y && my<=w.y+w.height as i32
        }).map(|w| w.id)
    }

    pub fn resize_edge_window_at(&self, mx: i32, my: i32) -> Option<(u32, ResizeEdge)> {
        self.windows.iter().rev().find_map(|w| {
            Self::resize_edge_at(w, mx, my).map(|e| (w.id, e))
        })
    }

    pub fn bring_to_front(&mut self, id: u32) {
        for w in &mut self.windows { w.active=false; }
        if let Some(idx) = self.windows.iter().position(|w| w.id==id) {
            let mut w = self.windows.remove(idx);
            w.active = true;
            self.windows.push(w);
        }
    }

    pub fn close_window(&mut self, id: u32) {
        self.windows.retain(|w| w.id!=id);
    }

    pub fn focused_id(&self) -> Option<u32> {
        self.windows.last().map(|w| w.id)
    }

    pub fn create_window(&mut self, title: String, app: String, width: u32, height: u32) {
        // Cascade new windows slightly
        let offset = (self.windows.len() % 8) as i32 * 28;
        let explorer = if app == "explorer" { Some(ExplorerState::new()) } else { None };
        let settings = if app == "settings" { Some(SettingsState::new()) } else { None };
        let w = Window {
            id: self.next_id,
            title, app,
            x: 120 + offset, y: 80 + offset,
            width, height,
            dragging: false, active: false,
            maximized: false, minimized: false,
            old_x: 120+offset, old_y: 80+offset,
            old_width: width, old_height: height,
            min_width: 200, min_height: 120,
            input_buffer:  String::new(),
            output_lines:  vec![
                "Trimangees OS Terminal v1.0".to_string(),
                "Type a command and press Enter. Tab to autocomplete.".to_string(),
                String::new(),
            ],
            cursor_blink:  true,
            scroll_offset: 0,
            history:       Vec::new(),
            history_index: None,
            pending_output: Vec::new(),
            running:        false,
            explorer,
            settings,
        };
        self.windows.push(w);
        self.next_id += 1;
    }

    pub fn list_windows(&self) {
        for w in &self.windows {
            println!("{} ({}) at ({},{})", w.title, w.id, w.x, w.y);
        }
    }
}
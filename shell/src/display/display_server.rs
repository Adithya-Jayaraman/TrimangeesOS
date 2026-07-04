use pixels::{Pixels, SurfaceTexture};
use crate::display::DesktopRenderer;
use crate::display::context_menu::ContextTarget;
use crate::window_manager::{WindowManager, SnapZone};
use crate::desktop_state::DesktopState;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
    keyboard::{Key, NamedKey},
    event::ElementState,
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

pub struct DisplayServer;

struct App {
    window: Option<&'static Window>,
    pixels: Option<Pixels<'static>>,
    desktop: DesktopState,

    // Modifier key state
    alt_pressed:   bool,
    shift_pressed: bool,

    // Dirty flag — only redraw when something actually changed
    needs_redraw:    bool,
    last_cursor_x:   i32,
    last_cursor_y:   i32,

    // Terminal cursor blink
    blink_counter:   u32,

    // Modifier state
    ctrl_pressed:    bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Trimangees OS")
                    .with_inner_size(LogicalSize::new(WIDTH, HEIGHT)),
            )
            .unwrap();

        let window = Box::leak(Box::new(window));

        let surface = SurfaceTexture::new(WIDTH, HEIGHT, &*window);

        let pixels = Pixels::new(WIDTH, HEIGHT, surface).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);

        // Welcome toast on startup
        self.desktop.notifications.push(
            "Trimangees OS",
            "Welcome! Right-click the desktop for options.",
        );

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.desktop.cursor.x = position.x as i32;
                self.desktop.cursor.y = position.y as i32;
                let mx = self.desktop.cursor.x;
                let my = self.desktop.cursor.y;

                // --- Active resize drag ---
                if let (Some(wid), Some(edge)) = (
                    self.desktop.resizing_window,
                    self.desktop.resize_edge,
                ) {
                    let smx = self.desktop.resize_start_mx;
                    let smy = self.desktop.resize_start_my;
                    let sw  = self.desktop.resize_start_w;
                    let sh  = self.desktop.resize_start_h;
                    let swx = self.desktop.resize_start_wx;
                    let swy = self.desktop.resize_start_wy;

                    if let Some(window) = self
                        .desktop.window_manager.windows
                        .iter_mut().find(|w| w.id == wid)
                    {
                        WindowManager::apply_resize(
                            window, edge,
                            smx, smy, sw, sh, swx, swy,
                            mx, my,
                        );
                    }

                    if let Some(wid) = self.desktop.resizing_window {
                        if let Some(win) = self.desktop.window_manager.windows.iter().find(|w| w.id == wid) {
                            self.desktop.webviews.sync_bounds(wid, win.x, win.y, win.width, win.height);
                        }
                    }

                    if let Some(w) = self.window { w.request_redraw(); }
                    return;
                }

                // --- Active window drag ---
                if let Some(window_id) = self.desktop.dragging_window {
                    if let Some(window) = self
                        .desktop.window_manager.windows
                        .iter_mut().find(|w| w.id == window_id)
                    {
                        window.x = mx - self.desktop.drag_offset_x;
                        window.y = my - self.desktop.drag_offset_y;
                        self.desktop.webviews.sync_bounds(window.id, window.x, window.y, window.width, window.height);
                    }
                    // Update snap preview
                    let new_snap = SnapZone::detect(mx, my);
                    if new_snap != self.desktop.snap_preview {
                        self.desktop.snap_preview = new_snap;
                    }
                    if let Some(w) = self.window { w.request_redraw(); }
                    return;
                }

                // --- Hover: update resize edge for cursor shape ---
                let prev_hover = self.desktop.hover_edge;
                self.desktop.hover_edge = self.desktop
                    .window_manager
                    .resize_edge_window_at(mx, my)
                    .map(|(_, e)| e);

                if self.desktop.hover_edge != prev_hover {
                    if let Some(w) = self.window { w.request_redraw(); }
                } else if let Some(w) = self.window {
                    w.request_redraw();
                }

                // Update start menu tile hover
                let prev_hovered = self.desktop.start_menu.hovered;
                self.desktop.start_menu.hovered = self.desktop.start_menu
                    .tile_at(mx, my, 720);
                let prev_power = self.desktop.start_menu.power_hovered;
                self.desktop.start_menu.power_hovered = self.desktop.start_menu
                    .power_btn_at(mx, my, 720);
                if self.desktop.start_menu.hovered != prev_hovered
                    || self.desktop.start_menu.power_hovered != prev_power {
                    if let Some(w) = self.window { w.request_redraw(); }
                }

                // Update context menu hover
                let prev_ctx = self.desktop.context_menu.hovered;
                self.desktop.context_menu.hovered = self.desktop.context_menu.item_at(mx, my);

                // Only redraw if something meaningful changed (avoids per-pixel redraws)
                let cursor_moved = (mx - self.last_cursor_x).abs() > 0
                    || (my - self.last_cursor_y).abs() > 0;
                let hover_changed = self.desktop.hover_edge.is_some()
                    || self.desktop.context_menu.hovered != prev_ctx
                    || self.desktop.start_menu.hovered != prev_hovered
                    || self.desktop.start_menu.power_hovered != prev_power;

                if cursor_moved || hover_changed {
                    self.last_cursor_x = mx;
                    self.last_cursor_y = my;
                    if let Some(w) = self.window { w.request_redraw(); }
                }
            }

            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {

                // ----------------------------------------
                // Context menu item click — handle first
                // ----------------------------------------
                if self.desktop.context_menu.visible {
                    let mx = self.desktop.cursor.x;
                    let my = self.desktop.cursor.y;

                    if let Some(idx) = self.desktop.context_menu.item_at(mx, my) {
                        let action = self.desktop.context_menu.items[idx].action.clone();
                        let target = self.desktop.context_menu.target.clone();
                        self.desktop.context_menu.close();

                        match action.as_str() {
                            "open_terminal" => {
                                self.desktop.application_manager.launch(
                                    "terminal", &mut self.desktop.window_manager);
                            }
                            "open_explorer" => {
                                self.desktop.application_manager.launch(
                                    "explorer", &mut self.desktop.window_manager);
                            }
                            "new_folder" => {
                                self.desktop.application_manager.launch(
                                    "explorer", &mut self.desktop.window_manager);
                            }
                            "close" => {
                                if let ContextTarget::Window(id) = target {
                                    self.desktop.window_manager.close_window(id);
                                }
                            }
                            "maximize" | "restore" => {
                                if let ContextTarget::Window(id) = target {
                                    self.desktop.window_manager.toggle_maximize(id);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        self.desktop.context_menu.close();
                    }

                    if let Some(w) = self.window { w.request_redraw(); }
                    return;
                }

                if let Some(id) = self
                    .desktop
                    .window_manager
                    .window_at(
                        self.desktop.cursor.x,
                        self.desktop.cursor.y,
                    )
                {

                    // ----------------------------------------
                    // Window button hit detection
                    // Mirrors the geometry in window_renderer.rs
                    // button_size=18, button_gap=4
                    // close  = x1 - 24 .. x1 - 6
                    // max    = x1 - 46 .. x1 - 28
                    // min    = x1 - 68 .. x1 - 50
                    // ----------------------------------------
                    if let Some(window) = self
                        .desktop
                        .window_manager
                        .windows
                        .iter()
                        .find(|w| w.id == id)
                    {
                        let cx     = self.desktop.cursor.x;
                        let cy     = self.desktop.cursor.y;
                        let x1     = window.x + window.width as i32;
                        let btn_y0 = window.y + 6;
                        let btn_y1 = window.y + 24;

                        // Close button
                        if cx >= x1 - 24 && cx <= x1 - 6
                            && cy >= btn_y0 && cy <= btn_y1
                        {
                            self.desktop.webviews.remove(id);
                            self.desktop.window_manager.close_window(id);
                            if let Some(w) = self.window { w.request_redraw(); }
                            return;
                        }

                        // Maximize / restore button
                        if cx >= x1 - 46 && cx <= x1 - 28
                            && cy >= btn_y0 && cy <= btn_y1
                        {
                            self.desktop.window_manager.toggle_maximize(id);
                            if let Some(w) = self.window { w.request_redraw(); }
                            return;
                        }

                        // Minimize button — minimise to taskbar
                        if cx >= x1 - 68 && cx <= x1 - 50
                            && cy >= btn_y0 && cy <= btn_y1
                        {
                            self.desktop.window_manager.minimize_window(id);
                            self.desktop.webviews.set_visible(id, false);
                            if let Some(w) = self.window { w.request_redraw(); }
                            return;
                        }

                        // Double-click on title bar → toggle maximize
                        if cy >= window.y && cy <= window.y + 30
                            && cx >= window.x && cx <= x1
                        {
                            let now = std::time::Instant::now();
                            let same_window = self.desktop.last_title_click_id == Some(id);
                            let double = same_window && self.desktop.last_title_click
                                .map(|t| now.duration_since(t).as_millis() < 400)
                                .unwrap_or(false);

                            if double {
                                self.desktop.window_manager.toggle_maximize(id);
                                self.desktop.last_title_click = None;
                                self.desktop.last_title_click_id = None;
                            } else {
                                self.desktop.last_title_click = Some(now);
                                self.desktop.last_title_click_id = Some(id);
                            }
                        }
                    }

                    // Check if clicking a resize edge first
                    if let Some(edge) = self.desktop.window_manager.windows
                        .iter().rev().find(|w| w.id == id)
                        .and_then(|w| WindowManager::resize_edge_at(w,
                            self.desktop.cursor.x,
                            self.desktop.cursor.y))
                    {
                        // Start resize
                        if let Some(window) = self.desktop.window_manager
                            .windows.iter().find(|w| w.id == id)
                        {
                            self.desktop.resizing_window  = Some(id);
                            self.desktop.resize_edge      = Some(edge);
                            self.desktop.resize_start_mx  = self.desktop.cursor.x;
                            self.desktop.resize_start_my  = self.desktop.cursor.y;
                            self.desktop.resize_start_wx  = window.x;
                            self.desktop.resize_start_wy  = window.y;
                            self.desktop.resize_start_w   = window.width;
                            self.desktop.resize_start_h   = window.height;
                        }
                        self.desktop.window_manager.bring_to_front(id);
                        return;
                    }

                    self.desktop
                        .window_manager
                        .bring_to_front(id);

                    if let Some(window) = self
                        .desktop
                        .window_manager
                        .windows
                        .iter_mut()
                        .find(|w| w.id == id)
                    {
                        self.desktop.drag_offset_x =
                            self.desktop.cursor.x - window.x;

                        self.desktop.drag_offset_y =
                            self.desktop.cursor.y - window.y;

                        window.dragging = true;
                    }

                    self.desktop.dragging_window = Some(id);

                    return;
                }
// ===============================
// Desktop icon selection
// ===============================

// ==========================
// Desktop icon selection
// ==========================

let now = Instant::now();

// Deselect everything
for icon in &mut self.desktop.desktop_icons {
    icon.selected = false;
}

for icon in &mut self.desktop.desktop_icons {

    let left = icon.x;
    let right = icon.x + 48;

    let top = icon.y;
    let bottom = icon.y + 48;

    if self.desktop.cursor.x >= left
        && self.desktop.cursor.x <= right
        && self.desktop.cursor.y >= top
        && self.desktop.cursor.y <= bottom
    {
        icon.selected = true;

        // Double-click detection
        if let Some(last) = icon.last_click {

            if now.duration_since(last)
                < Duration::from_millis(500)
            {
                self.desktop
    .application_manager
    .launch(
        &icon.app_name,
        &mut self.desktop.window_manager,
    );

                // Explorer launch comes in Lesson 51.
            }
        }

        icon.last_click = Some(now);

        if let Some(window) = self.window {
            window.request_redraw();
        }

        return;
    }
}
                // Body click — focus window + handle app-specific interactions
                if let Some(id) = self
                    .desktop
                    .window_manager
                    .window_body_at(
                        self.desktop.cursor.x,
                        self.desktop.cursor.y,
                    )
                {
                    self.desktop.window_manager.bring_to_front(id);

                    let cx = self.desktop.cursor.x;
                    let cy = self.desktop.cursor.y;

                    // Find the window and handle app-specific clicks
                    let app = self.desktop.window_manager.windows
                        .last().map(|w| w.app.clone()).unwrap_or_default();

                    match app.as_str() {
                        "explorer" => {
                            if let Some(win) = self.desktop.window_manager.windows.last_mut() {
                                let x0 = win.x as usize;
                                let y0 = win.y as usize;
                                let sidebar_w = 140usize;
                                let addr_h    = 28usize;
                                let title_h   = 33usize;
                                let row_h     = 22usize;
                                let content_x = x0 + sidebar_w + 1;
                                let list_y0   = y0 + title_h + addr_h + 21; // +header

                                if let Some(exp) = win.explorer.as_mut() {
                                    // Back button
                                    if cx as usize >= content_x+2 && (cx as usize) < content_x+26
                                    && cy as usize >= y0+title_h && (cy as usize) < y0+title_h+addr_h {
                                        exp.go_back();
                                    }
                                    // Forward button
                                    else if cx as usize >= content_x+26 && (cx as usize) < content_x+50
                                    && cy as usize >= y0+title_h && (cy as usize) < y0+title_h+addr_h {
                                        exp.go_forward();
                                    }
                                    // Up button
                                    else if cx as usize >= content_x+50 && (cx as usize) < content_x+80
                                    && cy as usize >= y0+title_h && (cy as usize) < y0+title_h+addr_h {
                                        exp.go_up();
                                    }
                                    // File row click
                                    else if cx as usize >= content_x && cy as usize >= list_y0 {
                                        let row = (cy as usize - list_y0) / row_h + exp.scroll_offset;
                                        if row < exp.entries.len() {
                                            if exp.selected == Some(row) {
                                                // Double-click: navigate into folder
                                                if exp.entries[row].is_dir {
                                                    let new_path = format!("{}/{}",
                                                        exp.current_path, exp.entries[row].name);
                                                    exp.navigate(new_path);
                                                } else {
                                                    // Open file with xdg-open
                                                    let path = format!("{}/{}",
                                                        exp.current_path, exp.entries[row].name);
                                                    let _ = std::process::Command::new("xdg-open")
                                                        .arg(&path).spawn();
                                                }
                                            } else {
                                                exp.selected = Some(row);
                                            }
                                        }
                                    }
                                    // Sidebar bookmark click
                                    else if cx as usize >= x0 && (cx as usize) < x0+sidebar_w {
                                        let bookmarks = [
                                            std::env::var("HOME").unwrap_or("/home".to_string()),
                                            format!("{}/Desktop", std::env::var("HOME").unwrap_or_default()),
                                            format!("{}/Documents",std::env::var("HOME").unwrap_or_default()),
                                            format!("{}/Downloads",std::env::var("HOME").unwrap_or_default()),
                                            "/".to_string(),
                                            "/etc".to_string(),
                                            "/usr/bin".to_string(),
                                        ];
                                        let bm_idx = (cy as usize - (y0+title_h)) / 22;
                                        if bm_idx < bookmarks.len() {
                                            let path = bookmarks[bm_idx].clone();
                                            exp.navigate(path);
                                        }
                                    }
                                }
                            }
                        }
                        "settings" => {
                            if let Some(win) = self.desktop.window_manager.windows.last_mut() {
                                let x0 = win.x as usize;
                                let y0 = win.y as usize;
                                let title_h   = 33usize;
                                let sidebar_w = 160usize;

                                if let Some(s) = win.settings.as_mut() {
                                    // Sidebar section click
                                    if cx as usize >= x0 && (cx as usize) < x0+sidebar_w {
                                        let sec = (cy as usize - (y0+title_h+12)) / 36;
                                        if sec < 3 { s.active_section = sec; }
                                    }
                                    // Wallpaper preset click (section 0)
                                    else if s.active_section == 0 {
                                        let px0 = x0+sidebar_w+16;
                                        let py0 = y0+title_h+16+50;
                                        if cy as usize >= py0 && (cy as usize) < py0+36 {
                                            let preset = ((cx as usize).saturating_sub(px0)) / 56;
                                            if preset < 4 {
                                                s.wallpaper_preset = preset as u8;
                                                // Invalidate wallpaper cache
                                                self.desktop.wallpaper_cache.clear();
                                            }
                                        }
                                        // Accent colour click
                                        let ay0 = y0+title_h+16+130;
                                        if cy as usize >= ay0 && (cy as usize) < ay0+28 {
                                            let accent_colors:[[u8;3];6]=[
                                                [72,138,240],[60,200,80],[220,100,60],
                                                [180,80,220],[240,180,40],[220,60,100]
                                            ];
                                            let ai = ((cx as usize).saturating_sub(px0)) / 36;
                                            if ai < 6 {
                                                s.accent_r = accent_colors[ai][0];
                                                s.accent_g = accent_colors[ai][1];
                                                s.accent_b = accent_colors[ai][2];
                                            }
                                        }
                                    }
                                    // Dark mode toggle (section 1)
                                    else if s.active_section == 1 {
                                        let tx = x0+sidebar_w+16+120;
                                        let ty = y0+title_h+16+28;
                                        if cx as usize >= tx && (cx as usize) < tx+44
                                        && cy as usize >= ty && (cy as usize) < ty+22 {
                                            s.dark_mode = !s.dark_mode;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    if let Some(w) = self.window { w.request_redraw(); }
                }

                // Start button
                if self.desktop.cursor.x >= 15
                    && self.desktop.cursor.x <= 45
                    && self.desktop.cursor.y >= (HEIGHT as i32 - 40)
                    && self.desktop.cursor.y <= (HEIGHT as i32 - 10)
                {
                    self.desktop.start_menu.toggle();

                    if let Some(window) = self.window {
                        window.request_redraw();
                    }
                    return;
                }

                // ----------------------------------------
                // Toast dismiss on click
                // ----------------------------------------
                if self.desktop.notifications.dismiss_at(
                    self.desktop.cursor.x,
                    self.desktop.cursor.y,
                    1280, 720,
                ) {
                    if let Some(w) = self.window { w.request_redraw(); }
                    return;
                }

                // ----------------------------------------
                // Taskbar app button clicks (running windows)
                // Buttons start at WIDTH/2+70, each 110px wide + 6px gap
                // ----------------------------------------
                {
                    let btn_start_x = 1280 / 2 + 70;
                    let btn_w       = 110i32;
                    let btn_gap     = 6i32;
                    let btn_h       = 32i32;
                    let btn_y0_t    = 720i32 - 48 + (48 - btn_h) / 2;
                    let btn_y1_t    = btn_y0_t + btn_h;
                    let cy_t        = self.desktop.cursor.y;
                    let cx_t        = self.desktop.cursor.x;

                    if cy_t >= btn_y0_t && cy_t <= btn_y1_t {
                        let wins_len = self.desktop.window_manager.windows.len() as i32;
                        for i in 0..wins_len {
                            let bx = btn_start_x as i32 + i * (btn_w + btn_gap);
                            if bx + btn_w > 1280 - 120 { break; }

                            if cx_t >= bx && cx_t <= bx + btn_w {
                                let win_id = self.desktop.window_manager.windows[i as usize].id;
                                let is_minimized = self.desktop.window_manager.windows[i as usize].minimized;
                                let is_active    = self.desktop.window_manager.windows[i as usize].active;

                                if is_minimized {
                                    // Restore minimised window
                                    self.desktop.window_manager.restore_window(win_id);
                                    self.desktop.webviews.set_visible(win_id, true);
                                } else if is_active {
                                    // Click active window button → minimise it
                                    self.desktop.window_manager.minimize_window(win_id);
                                } else {
                                    // Focus a background window
                                    self.desktop.window_manager.bring_to_front(win_id);
                                }

                                if let Some(w) = self.window { w.request_redraw(); }
                                return;
                            }
                        }
                    }
                }

                // ----------------------------------------
                // Taskbar unified icon buttons — centred, Windows 11 style
                // Mirrors draw_taskbar_buttons layout exactly
                // ----------------------------------------
                {
                    const TB: usize = 36;
                    let btn_size = TB - 4;
                    let gap      = 4usize;
                    let n_pinned = 6usize; // must match pinned array length
                    let total_w  = n_pinned * (btn_size + gap) - gap;
                    let start_x  = (1280 - total_w) / 2;
                    let by0      = 720 - TB;
                    let by1      = 720usize;

                    let cx2 = self.desktop.cursor.x as usize;
                    let cy2 = self.desktop.cursor.y as usize;

                    let app_ids = ["explorer","browser","terminal","tridocs","trisheets","trislides"];

                    if cy2 >= by0 && cy2 < by1 {
                        for (pi, &app_id) in app_ids.iter().enumerate() {
                            let bx = start_x + pi * (btn_size + gap);
                            if cx2 >= bx && cx2 < bx + btn_size {
                                // Find open windows for this app
                                let open: Vec<u32> = self.desktop.window_manager.windows
                                    .iter()
                                    .filter(|w| w.app == app_id)
                                    .map(|w| w.id)
                                    .collect();

                                if open.is_empty() {
                                    // No open window — launch
                                    self.desktop.application_manager.launch(
                                        app_id, &mut self.desktop.window_manager);
                                } else {
                                    // Has open windows — cycle focus or restore
                                    let active = self.desktop.window_manager.windows
                                        .iter().find(|w| w.app == app_id && w.active && !w.minimized);
                                    if active.is_some() {
                                        // Already active — minimise it
                                        let id = *open.last().unwrap();
                                        self.desktop.window_manager.minimize_window(id);
                                    } else {
                                        // Restore/focus top window for this app
                                        let id = *open.last().unwrap();
                                        self.desktop.window_manager.restore_window(id);
                                    }
                                }
                                if let Some(w) = self.window { w.request_redraw(); }
                                return;
                            }
                        }
                    }
                }

                // Start menu tile / power click — launch app
                if self.desktop.start_menu.visible {
                    let mx2 = self.desktop.cursor.x;
                    let my2 = self.desktop.cursor.y;

                    // Power buttons
                    if let Some(power_idx) = self.desktop.start_menu.power_btn_at(mx2, my2, 720) {
                        self.desktop.start_menu.visible = false;
                        match power_idx {
                            0 => { let _ = std::process::Command::new("systemctl").arg("suspend").spawn(); }
                            1 => { let _ = std::process::Command::new("shutdown").args(["-r","now"]).spawn(); }
                            2 => { let _ = std::process::Command::new("shutdown").args(["now"]).spawn(); }
                            _ => {}
                        }
                        if let Some(w) = self.window { w.request_redraw(); }
                        return;
                    }

                    if let Some(tile_idx) = self.desktop.start_menu
                        .tile_at(self.desktop.cursor.x, self.desktop.cursor.y, 720)
                    {
                        let executable = self.desktop.start_menu
                            .entries[tile_idx].executable.clone();

                        self.desktop.application_manager.launch(
                            &executable,
                            &mut self.desktop.window_manager,
                        );

                        if let Some(shell_win) = self.window {
                            if let Some(new_win) = self.desktop.window_manager.windows.last() {
                                let app = new_win.app.clone();
                                let wid = new_win.id;
                                let (wx,wy,ww,wh) = (new_win.x,new_win.y,new_win.width,new_win.height);
                                match app.as_str() {
                                    "browser"|"tridocs"|"trisheets"|"trislides"|"tridraw" => {
                                        self.desktop.webviews.open(shell_win,wid,&app,wx,wy,ww,wh);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        self.desktop.start_menu.visible = false;
                        self.desktop.start_menu.hovered = None;

                        if let Some(w) = self.window { w.request_redraw(); }
                        return;
                    }

                    // Click outside menu — close it
                    self.desktop.start_menu.visible = false;
                    if let Some(w) = self.window { w.request_redraw(); }
                }
            }

            WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                // Stop resize
                self.desktop.resizing_window = None;
                self.desktop.resize_edge     = None;

                // Apply snap if preview is active
                if let (Some(wid), Some(zone)) = (
                    self.desktop.dragging_window,
                    self.desktop.snap_preview,
                ) {
                    self.desktop.window_manager.snap_window(wid, zone);
                }
                self.desktop.snap_preview = None;

                // Stop drag
                if let Some(window_id) = self.desktop.dragging_window {
                    if let Some(window) = self
                        .desktop
                        .window_manager
                        .windows
                        .iter_mut()
                        .find(|w| w.id == window_id)
                    {
                        window.dragging = false;
                    }
                }

                self.desktop.dragging_window = None;

                if let Some(window) = self.window {
                    window.request_redraw();
                }
            }

            // ----------------------------------------
            // Right-click — open context menu
            // ----------------------------------------
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Right,
                ..
            } => {
                let mx = self.desktop.cursor.x;
                let my = self.desktop.cursor.y;

                // Close any open menu first
                self.desktop.context_menu.close();

                // Check if clicking on a window title bar
                if let Some(id) = self.desktop.window_manager.window_at(mx, my) {
                    self.desktop.context_menu.open_window(mx, my, id, 1280, 720);
                } else {
                    self.desktop.context_menu.open_desktop(mx, my, 1280, 720);
                }

                if let Some(w) = self.window { w.request_redraw(); }
            }

            // ----------------------------------------
            // Right-click release — handle menu action
            // ----------------------------------------
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Right,
                ..
            } => {
                // nothing on release — action fires on left-click of item
            }


            // ── Mouse wheel — scroll terminal or explorer ──────────

            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                let lines_delta: i32 = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -(y as i32),
                    MouseScrollDelta::PixelDelta(pos) => -(pos.y as i32 / 16),
                };

                // Find topmost terminal window under cursor
                let _cx = self.desktop.cursor.x;
                let _cy = self.desktop.cursor.y;

                let cx2 = self.desktop.cursor.x;
                let cy2 = self.desktop.cursor.y;
                // Find topmost window under cursor
                if let Some(win) = self.desktop.window_manager.windows
                    .iter_mut().rev()
                    .find(|w| !w.minimized
                        && cx2 >= w.x && cx2 <= w.x + w.width  as i32
                        && cy2 >= w.y && cy2 <= w.y + w.height as i32)
                {
                    match win.app.as_str() {
                        "terminal" => {
                            let max_scroll = win.output_lines.len().saturating_sub(3);
                            if lines_delta > 0 {
                                win.scroll_offset = (win.scroll_offset + lines_delta as usize).min(max_scroll);
                            } else {
                                win.scroll_offset = win.scroll_offset.saturating_sub((-lines_delta) as usize);
                            }
                        }
                        "explorer" => {
                            if let Some(exp) = win.explorer.as_mut() {
                                let max_scroll = exp.entries.len().saturating_sub(5);
                                if lines_delta > 0 {
                                    exp.scroll_offset = (exp.scroll_offset + lines_delta as usize).min(max_scroll);
                                } else {
                                    exp.scroll_offset = exp.scroll_offset.saturating_sub((-lines_delta) as usize);
                                }
                            }
                        }
                        _ => {}
                    }
                    if let Some(w) = self.window { w.request_redraw(); }
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    let frame = pixels.frame_mut();

                    DesktopRenderer::draw(
                        frame,
                        &mut self.desktop,
                    );

                    pixels.render().unwrap();
                }
            }

            // ----------------------------------------
            // Modifier tracking (Alt, Shift)
            // ----------------------------------------
            WindowEvent::ModifiersChanged(mods) => {
                let alt_was = self.alt_pressed;
                self.alt_pressed   = mods.state().alt_key();
                self.shift_pressed = mods.state().shift_key();
                self.ctrl_pressed  = mods.state().control_key();

                // Alt released — commit alt+tab selection
                if alt_was && !self.alt_pressed && self.desktop.alt_tab_visible {
                    self.desktop.alt_tab_visible = false;
                    // Find the selected visible window and focus it
                    let idx = self.desktop.alt_tab_index;
                    let visible_ids: Vec<u32> = self.desktop.window_manager.windows
                        .iter()
                        .filter(|w| !w.minimized)
                        .map(|w| w.id)
                        .collect();
                    if let Some(&id) = visible_ids.get(idx) {
                        self.desktop.window_manager.bring_to_front(id);
                    }
                    if let Some(w) = self.window { w.request_redraw(); }
                }
            }

            // ----------------------------------------
            // Keyboard shortcuts
            // ----------------------------------------
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    // only act on key-down
                } else {
                    // ── Route to focused terminal window first ──────────
                    let focused_is_terminal = self.desktop.window_manager.windows
                        .last()
                        .map(|w| w.app == "terminal" && !w.minimized)
                        .unwrap_or(false);

                    if focused_is_terminal {
                        if let Some(win) = self.desktop.window_manager.windows.last_mut() {
                            match &event.logical_key {
                                Key::Named(NamedKey::Backspace) => {
                                    win.input_buffer.pop();
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }
                                Key::Named(NamedKey::Enter) => {
                                    let cmd = win.input_buffer.trim().to_string();

                                    // Echo the prompt line to output
                                    win.output_lines.push(
                                        format!("user@trimangees:~$ {}", cmd)
                                    );

                                    // Add to history (skip empty and duplicates)
                                    if !cmd.is_empty() {
                                        if win.history.last().map(|l| l != &cmd).unwrap_or(true) {
                                            win.history.push(cmd.clone());
                                        }
                                    }
                                    win.history_index = None;
                                    win.input_buffer.clear();
                                    win.scroll_offset = 0;

                                    if cmd.is_empty() {
                                        win.output_lines.push(String::new());
                                        if let Some(w) = self.window { w.request_redraw(); }
                                        return;
                                    }

                                    // Built-in commands
                                    match cmd.as_str() {
                                        "clear" => {
                                            win.output_lines.clear();
                                            win.output_lines.push(String::new());
                                            if let Some(w) = self.window { w.request_redraw(); }
                                            return;
                                        }
                                        "exit" | "quit" => {
                                            let id = win.id;
                                            drop(win); // release borrow
                                            self.desktop.window_manager.close_window(id);
                                            if let Some(w) = self.window { w.request_redraw(); }
                                            return;
                                        }
                                        _ => {}
                                    }

                                    // Execute via sh -c
                                    // On Linux this runs the real shell.
                                    // On Windows dev builds it will fail gracefully.
                                    let result = std::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(&cmd)
                                        .output();

                                    match result {
                                        Ok(out) => {
                                            // Push stdout lines
                                            let stdout = String::from_utf8_lossy(&out.stdout);
                                            for line in stdout.lines() {
                                                win.output_lines.push(line.to_string());
                                            }
                                            // Push stderr lines (dimmed)
                                            let stderr = String::from_utf8_lossy(&out.stderr);
                                            for line in stderr.lines() {
                                                win.output_lines.push(
                                                    format!("stderr: {}", line)
                                                );
                                            }
                                            if !out.status.success() {
                                                win.output_lines.push(
                                                    format!("exit code: {}", out.status.code().unwrap_or(-1))
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            // sh not found (Windows dev build)
                                            win.output_lines.push(
                                                format!("error: {}", e)
                                            );
                                        }
                                    }

                                    win.output_lines.push(String::new());
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }
                                Key::Named(NamedKey::ArrowUp) => {
                                    // Command history recall
                                    if !win.history.is_empty() {
                                        let new_idx = match win.history_index {
                                            None => win.history.len() - 1,
                                            Some(i) => i.saturating_sub(1),
                                        };
                                        win.history_index = Some(new_idx);
                                        win.input_buffer = win.history[new_idx].clone();
                                    }
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }
                                Key::Named(NamedKey::ArrowDown) => {
                                    // Scroll forward through history
                                    match win.history_index {
                                        Some(i) if i + 1 < win.history.len() => {
                                            win.history_index = Some(i + 1);
                                            win.input_buffer = win.history[i+1].clone();
                                        }
                                        _ => {
                                            win.history_index = None;
                                            win.input_buffer.clear();
                                        }
                                    }
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }
                                Key::Named(NamedKey::Escape) => {
                                    // Escape clears input (don't close menu)
                                    win.input_buffer.clear();
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }
                                Key::Named(NamedKey::Tab) => {
                                    // Tab autocomplete — complete the last word
                                    // using entries from the current directory
                                    let buf = win.input_buffer.clone();
                                    let last_word = buf.split_whitespace().last().unwrap_or("").to_string();
                                    let prefix = if last_word.is_empty() { "".to_string() }
                                        else { last_word.clone() };

                                    // Get completions from filesystem
                                    let search_dir = if prefix.contains('/') {
                                        let p = std::path::Path::new(&prefix);
                                        p.parent().unwrap_or(std::path::Path::new("."))
                                            .to_string_lossy().to_string()
                                    } else {
                                        ".".to_string()
                                    };
                                    let file_prefix = prefix.split('/').last().unwrap_or(&prefix);

                                    let mut matches: Vec<String> = Vec::new();
                                    if let Ok(entries) = std::fs::read_dir(&search_dir) {
                                        for entry in entries.flatten() {
                                            let name = entry.file_name().to_string_lossy().to_string();
                                            if name.starts_with(file_prefix) {
                                                let suffix = if entry.path().is_dir() { "/" } else { "" };
                                                matches.push(format!("{}{}", name, suffix));
                                            }
                                        }
                                    }
                                    matches.sort();

                                    if matches.len() == 1 {
                                        // Single match — complete it
                                        let completed = &matches[0];
                                        let new_buf = if last_word.is_empty() {
                                            format!("{}{}", buf, completed)
                                        } else {
                                            buf[..buf.len()-last_word.len()].to_string() + completed
                                        };
                                        win.input_buffer = new_buf;
                                    } else if matches.len() > 1 {
                                        // Multiple matches — show them
                                        win.output_lines.push(
                                            format!("user@trimangees:~$ {}", win.input_buffer)
                                        );
                                        win.output_lines.push(matches.join("  "));
                                        win.output_lines.push(String::new());
                                    }
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }

                                Key::Character(s) => {
                                    // Skip Ctrl+key combinations (already handled above)
                                    if self.ctrl_pressed { return; }
                                    // Append printable character to input buffer
                                    for ch in s.chars() {
                                        if !ch.is_control() {
                                            win.input_buffer.push(ch);
                                        }
                                    }
                                    if let Some(w) = self.window { w.request_redraw(); }
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }

                    // ── Global shortcuts (only when terminal not consuming input) ──
                    match &event.logical_key {

                        // Super / Win key — toggle start menu
                        Key::Named(NamedKey::Super) => {
                            self.desktop.start_menu.toggle();
                            if let Some(w) = self.window { w.request_redraw(); }
                        }

                        // Alt+F4 — close the focused (top) window
                        Key::Named(NamedKey::F4) if self.alt_pressed => {
                            if let Some(window) = self.desktop.window_manager.windows.last() {
                                let id = window.id;
                                self.desktop.window_manager.close_window(id);
                                if let Some(w) = self.window { w.request_redraw(); }
                            }
                        }

                        // Alt+Tab — show visual switcher, advance selection
                        Key::Named(NamedKey::Tab) if self.alt_pressed => {
                            let len = self.desktop.window_manager.windows
                                .iter().filter(|w| !w.minimized).count();
                            if len > 0 {
                                if !self.desktop.alt_tab_visible {
                                    self.desktop.alt_tab_visible = true;
                                    self.desktop.alt_tab_index   = 0;
                                }
                                if self.shift_pressed {
                                    self.desktop.alt_tab_index =
                                        self.desktop.alt_tab_index.wrapping_sub(1)
                                        % len.max(1);
                                } else {
                                    self.desktop.alt_tab_index =
                                        (self.desktop.alt_tab_index + 1) % len.max(1);
                                }
                                if let Some(w) = self.window { w.request_redraw(); }
                            }
                        }

                        // Ctrl+C — interrupt current input / running command
                        Key::Character(s) if (s == "c" || s == "C") && self.ctrl_pressed => {
                            if let Some(win) = self.desktop.window_manager.windows
                                .iter_mut().last()
                                .filter(|w| w.app == "terminal" && !w.minimized)
                            {
                                let _had_input = !win.input_buffer.is_empty();
                                win.output_lines.push(
                                    format!("user@trimangees:~$ {}^C", win.input_buffer)
                                );
                                win.output_lines.push(String::new());
                                win.input_buffer.clear();
                                win.scroll_offset = 0;
                                if let Some(w) = self.window { w.request_redraw(); }
                            }
                            return;
                        }

                        // Ctrl+D — EOF / close terminal
                        Key::Character(s) if (s == "d" || s == "D") && self.ctrl_pressed => {
                            if let Some(win) = self.desktop.window_manager.windows
                                .last()
                                .filter(|w| w.app == "terminal" && !w.minimized && w.input_buffer.is_empty())
                            {
                                let id = win.id;
                                self.desktop.window_manager.close_window(id);
                                if let Some(w) = self.window { w.request_redraw(); }
                            } else if let Some(win) = self.desktop.window_manager.windows
                                .iter_mut().last()
                                .filter(|w| w.app == "terminal" && !w.minimized)
                            {
                                // If input buffer not empty, just clear it
                                win.input_buffer.clear();
                                if let Some(w) = self.window { w.request_redraw(); }
                            }
                            return;
                        }

                        // Ctrl+L — clear screen (like shell)
                        Key::Character(s) if (s == "l" || s == "L") && self.ctrl_pressed => {
                            if let Some(win) = self.desktop.window_manager.windows
                                .iter_mut().last()
                                .filter(|w| w.app == "terminal" && !w.minimized)
                            {
                                win.output_lines.clear();
                                win.output_lines.push(String::new());
                                win.scroll_offset = 0;
                                if let Some(w) = self.window { w.request_redraw(); }
                            }
                            return;
                        }

                        // Escape — close start menu or context menu
                        Key::Named(NamedKey::Escape) => {
                            let mut changed = false;
                            if self.desktop.start_menu.visible {
                                self.desktop.start_menu.visible = false;
                                changed = true;
                            }
                            if self.desktop.context_menu.visible {
                                self.desktop.context_menu.close();
                                changed = true;
                            }
                            if changed {
                                if let Some(w) = self.window { w.request_redraw(); }
                            }
                        }

                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Toast expiry
        if self.desktop.notifications.has_toasts() {
            let changed = self.desktop.notifications.tick();
            if changed {
                if let Some(w) = self.window { w.request_redraw(); }
            }
        }

        // Terminal cursor blink — every ~30 about_to_wait calls
        self.blink_counter = self.blink_counter.wrapping_add(1);
        if self.blink_counter % 30 == 0 {
            let has_terminal = self.desktop.window_manager.windows
                .iter().any(|w| w.app == "terminal" && w.active && !w.minimized);
            if has_terminal {
                for win in self.desktop.window_manager.windows.iter_mut() {
                    if win.app == "terminal" {
                        win.cursor_blink = !win.cursor_blink;
                    }
                }
                if let Some(w) = self.window { w.request_redraw(); }
            }
        }
    }
}

impl DisplayServer {
    pub fn run() {
        let event_loop = EventLoop::new().unwrap();

        let mut app = App {
            window: None,
            pixels: None,
            desktop: DesktopState::new(),
            alt_pressed:   false,
            shift_pressed: false,
            needs_redraw:   true,
            last_cursor_x:  0,
            last_cursor_y:  0,
            blink_counter:  0,
            ctrl_pressed:   false,
        };

        event_loop.run_app(&mut app).unwrap();
    }

}
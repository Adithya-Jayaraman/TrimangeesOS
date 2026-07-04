use crate::application_manager::ApplicationManager;
use crate::display::ContextMenu;
use crate::display::FontRenderer;
use crate::display::NotificationManager;
use crate::window_manager::ResizeEdge;
use crate::window_manager::SnapZone;
use std::time::Instant;
use crate::display::{Cursor, StartMenu};
use crate::window_manager::WindowManager;
use crate::desktop_icon::DesktopIcon;
use crate::webview_manager::WebViewManager;

pub struct DesktopState {
    pub cursor:              Cursor,
    pub context_menu:        ContextMenu,
    pub font:                FontRenderer,
    pub wallpaper_cache:     Vec<u8>,
    pub notifications:       NotificationManager,
    pub start_menu:          StartMenu,
    pub window_manager:      WindowManager,
    pub application_manager: ApplicationManager,
    pub desktop_icons:       Vec<DesktopIcon>,
    pub webviews:            WebViewManager,

    pub dragging_window:     Option<u32>,
    pub drag_offset_x:       i32,
    pub drag_offset_y:       i32,

    pub last_title_click:    Option<Instant>,
    pub last_title_click_id: Option<u32>,

    pub resizing_window:     Option<u32>,
    pub resize_edge:         Option<ResizeEdge>,
    pub resize_start_mx:     i32,
    pub resize_start_my:     i32,
    pub resize_start_wx:     i32,
    pub resize_start_wy:     i32,
    pub resize_start_w:      u32,
    pub resize_start_h:      u32,
    pub hover_edge:          Option<ResizeEdge>,
    pub snap_preview:        Option<SnapZone>,

    pub alt_tab_visible:     bool,
    pub alt_tab_index:       usize,
}

impl DesktopState {
    pub fn new() -> Self {
        let mut window_manager = WindowManager::new();
        window_manager.create_window(
            "Terminal".to_string(), "terminal".to_string(), 700, 400,
        );
        let desktop_icons = vec![
            DesktopIcon::new("File Explorer", "explorer", 40, 40),
        ];
        Self {
            cursor:               Cursor::new(),
            context_menu:         ContextMenu::new(),
            font:                 FontRenderer::new(),
            wallpaper_cache:      Vec::new(),
            notifications:        NotificationManager::new(),
            start_menu:           StartMenu::new(),
            window_manager,
            application_manager:  ApplicationManager::new(),
            desktop_icons,
            webviews:             WebViewManager::new(),
            dragging_window:      None,
            drag_offset_x:        0,
            drag_offset_y:        0,
            last_title_click:     None,
            last_title_click_id:  None,
            resizing_window:      None,
            resize_edge:          None,
            resize_start_mx:      0,
            resize_start_my:      0,
            resize_start_wx:      0,
            resize_start_wy:      0,
            resize_start_w:       0,
            resize_start_h:       0,
            hover_edge:           None,
            snap_preview:         None,
            alt_tab_visible:      false,
            alt_tab_index:        0,
        }
    }
}
use wry::{WebView, WebViewBuilder, Rect};
use wry::dpi::{LogicalPosition, LogicalSize};
use std::collections::HashMap;

const DOCS_HTML:    &str = include_str!("../assets/apps/docs.html");
const SHEETS_HTML:  &str = include_str!("../assets/apps/sheets.html");
const SLIDES_HTML:  &str = include_str!("../assets/apps/slides.html");
const DRAWING_HTML: &str = include_str!("../assets/apps/drawing.html");
const BROWSER_HTML: &str = include_str!("../assets/apps/browser.html");

const TITLE_H: i32 = 33;

pub struct WebViewManager {
    views: HashMap<u32, WebView>,
}

impl WebViewManager {
    pub fn new() -> Self { Self { views: HashMap::new() } }

    pub fn open(
        &mut self,
        parent: &winit::window::Window,
        window_id: u32,
        app: &str,
        wx: i32, wy: i32, ww: u32, wh: u32,
    ) {
        let html: &str = match app {
            "tridocs"   => DOCS_HTML,
            "trisheets" => SHEETS_HTML,
            "trislides" => SLIDES_HTML,
            "tridraw"   => DRAWING_HTML,
            "browser"   => BROWSER_HTML,
            _           => return,
        };
        self.views.remove(&window_id);
        let init = r#"window.__TRIMANGEES_OS__=true;window.__TRIMANGEES_VERSION__='1.0';
window.trimangees={
    notify:function(t,b){console.log('__NOTIFY__:'+t+':'+(b||''));},
    closeWindow:function(){console.log('__CLOSE__');}
};"#;
        match WebViewBuilder::new_as_child(parent)
            .with_bounds(Rect {
                position: LogicalPosition::new(wx, wy + TITLE_H).into(),
                size:     LogicalSize::new(ww, wh.saturating_sub(TITLE_H as u32)).into(),
            })
            .with_html(html)
            .with_initialization_script(init)
            .build()
        {
            Ok(wv)  => { self.views.insert(window_id, wv); }
            Err(e)  => { eprintln!("[wry] {}: {}", app, e); }
        }
    }

    pub fn sync_bounds(&self, id: u32, wx: i32, wy: i32, ww: u32, wh: u32) {
        if let Some(wv) = self.views.get(&id) {
            let _ = wv.set_bounds(Rect {
                position: LogicalPosition::new(wx, wy + TITLE_H).into(),
                size:     LogicalSize::new(ww, wh.saturating_sub(TITLE_H as u32)).into(),
            });
        }
    }

    pub fn set_visible(&self, id: u32, v: bool) {
        if let Some(wv) = self.views.get(&id) { let _ = wv.set_visible(v); }
    }

    pub fn remove(&mut self, id: u32) { self.views.remove(&id); }
    pub fn has(&self, id: u32) -> bool { self.views.contains_key(&id) }
}
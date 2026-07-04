// ============================================================
// NotificationManager — toast pop-ups, bottom-right corner
// ============================================================
use std::time::Instant;

pub const TOAST_W:        usize = 280;
pub const TOAST_H:        usize = 60;
pub const TOAST_PAD_X:    usize = 16;  // from right edge of screen
pub const TOAST_PAD_Y:    usize = 64;  // from bottom (above taskbar)
pub const TOAST_GAP:      usize = 8;   // between stacked toasts
pub const TOAST_LIFETIME: u64   = 3;   // seconds before auto-dismiss

pub struct Toast {
    pub id:        u32,
    pub title:     String,
    pub body:      String,
    pub created:   Instant,
    pub dismissed: bool,
}

pub struct NotificationManager {
    pub toasts:  Vec<Toast>,
    next_id:     u32,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self { toasts: Vec::new(), next_id: 1 }
    }

    /// Push a new toast. Max 4 visible at once (oldest dropped).
    pub fn push(&mut self, title: &str, body: &str) {
        if self.toasts.len() >= 4 {
            self.toasts.remove(0);
        }
        self.toasts.push(Toast {
            id:        self.next_id,
            title:     title.to_string(),
            body:      body.to_string(),
            created:   Instant::now(),
            dismissed: false,
        });
        self.next_id += 1;
    }

    /// Expire toasts older than TOAST_LIFETIME seconds.
    /// Returns true if any toast was removed (caller should redraw).
    pub fn tick(&mut self) -> bool {
        let before = self.toasts.len();
        self.toasts.retain(|t| {
            !t.dismissed
                && t.created.elapsed().as_secs() < TOAST_LIFETIME
        });
        self.toasts.len() != before
    }

    /// Dismiss the toast at screen position (mx, my).
    /// Returns true if a toast was hit.
    pub fn dismiss_at(&mut self, mx: i32, my: i32, screen_w: usize, screen_h: usize) -> bool {
        for (i, toast) in self.toasts.iter_mut().enumerate() {
            let (tx, ty) = Self::toast_pos(i, screen_w, screen_h);
            if mx >= tx && mx <= tx + TOAST_W as i32
            && my >= ty && my <= ty + TOAST_H as i32 {
                toast.dismissed = true;
                return true;
            }
        }
        false
    }

    /// Pixel position of toast at stack index i (bottom-right, stacked upward)
    pub fn toast_pos(i: usize, screen_w: usize, screen_h: usize) -> (i32, i32) {
        let x = (screen_w - TOAST_W - TOAST_PAD_X) as i32;
        let y = (screen_h - TOAST_PAD_Y - TOAST_H - i * (TOAST_H + TOAST_GAP)) as i32;
        (x, y)
    }

    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }
}
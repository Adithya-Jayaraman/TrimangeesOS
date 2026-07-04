// ============================================================
// FontRenderer — ab_glyph powered text using Inter font
// Replaces TextRenderer for all visible UI text
// ============================================================
use ab_glyph::{FontRef, PxScale, Font, ScaleFont};

pub struct FontRenderer {
    pub font: FontRef<'static>,
}

const SW: usize = 1280;
const SH: usize = 720;

// Font bytes embedded at compile time — zero runtime file I/O
static FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");

impl FontRenderer {
    pub fn new() -> Self {
        let font = FontRef::try_from_slice(FONT_DATA)
            .expect("Inter-Regular.ttf failed to parse — check assets/fonts/ exists");
        Self { font }
    }

    /// Draw text at (x, y) with given size in pixels and colour [R,G,B]
    pub fn draw(
        &self,
        frame: &mut [u8],
        x: usize,
        y: usize,
        text: &str,
        size: f32,
        color: [u8; 3],
    ) {
        // Skip drawing if totally off-screen
        if y >= SH || x >= SW { return; }
        if size <= 0.0 || size > 200.0 { return; }

        let scale  = PxScale::from(size);
        let scaled = self.font.as_scaled(scale);
        let mut cx = x as f32;

        for ch in text.chars() {
            let glyph_id = scaled.glyph_id(ch);
            let glyph    = glyph_id.with_scale_and_position(scale, ab_glyph::point(cx, y as f32 + scaled.ascent()));

            if let Some(outlined) = self.font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    // Stay in f32 the entire time — no integer overflow possible
                    let px_f = bounds.min.x + gx as f32;
                    let py_f = bounds.min.y + gy as f32;
                    // Strict bounds check before any cast
                    if px_f < 0.0 || py_f < 0.0 { return; }
                    if px_f >= (SW - 1) as f32 || py_f >= (SH - 1) as f32 { return; }
                    let px = px_f as usize;
                    let py = py_f as usize;
                    // Double-check after cast (NaN safety)
                    if px >= SW || py >= SH { return; }
                    let idx = py * SW + px;
                    if idx * 4 + 3 >= frame.len() { return; }
                    let i = idx * 4;
                    let a = (cov.clamp(0.0, 1.0) * 255.0) as u16;
                    let ia = 255u16 - a;
                    frame[i]   = ((color[0] as u16 * a + frame[i]   as u16 * ia) / 255) as u8;
                    frame[i+1] = ((color[1] as u16 * a + frame[i+1] as u16 * ia) / 255) as u8;
                    frame[i+2] = ((color[2] as u16 * a + frame[i+2] as u16 * ia) / 255) as u8;
                });
            }

            cx += scaled.h_advance(glyph_id);
        }
    }

    /// Draw text centred horizontally in [x0, x1]
    pub fn draw_centered(
        &self,
        frame: &mut [u8],
        x0: usize,
        x1: usize,
        y: usize,
        text: &str,
        size: f32,
        color: [u8; 3],
    ) {
        let w = self.measure(text, size) as usize;
        let x = x0 + (x1.saturating_sub(x0)).saturating_sub(w) / 2;
        self.draw(frame, x, y, text, size, color);
    }

    /// Measure pixel width of text at given size
    pub fn measure(&self, text: &str, size: f32) -> f32 {
        let scale  = PxScale::from(size);
        let scaled = self.font.as_scaled(scale);
        text.chars().map(|c| scaled.h_advance(scaled.glyph_id(c))).sum()
    }

    /// Truncate text to fit within max_px pixels, appending ".." if needed
    pub fn truncate(&self, text: &str, size: f32, max_px: f32) -> String {
        let ellipsis_w = self.measure("..", size);
        let mut out = String::new();
        let mut used = 0.0f32;
        let chars: Vec<char> = text.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            let cw = {
                let scale  = PxScale::from(size);
                let scaled = self.font.as_scaled(scale);
                scaled.h_advance(scaled.glyph_id(ch))
            };
            let remaining = chars.len() - i;
            let need = if remaining > 1 { ellipsis_w } else { 0.0 };
            if used + cw > (max_px - need).max(0.0) && remaining > 1 {
                out.push_str("..");
                break;
            }
            out.push(ch);
            used += cw;
        }
        out
    }
}
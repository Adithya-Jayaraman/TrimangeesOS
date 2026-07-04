use crate::desktop_icon::DesktopIcon;
use crate::display::TextRenderer;

pub struct IconRenderer;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

impl IconRenderer {
    pub fn draw(
        frame: &mut [u8],
        icons: &[DesktopIcon],
    ) {
        for icon in icons {
            Self::draw_icon(frame, icon);
        }
    }

    fn draw_icon(
        frame: &mut [u8],
        icon: &DesktopIcon,
    ) {
        let x = icon.x as usize;
        let y = icon.y as usize;

        // ==========================
        // Selection Highlight (NEW)
        // ==========================
        if icon.selected {
            for yy in y.saturating_sub(4)..(y + 52) {
                for xx in x.saturating_sub(4)..(x + 52) {
                    if xx >= WIDTH || yy >= HEIGHT {
                        continue;
                    }

                    let i = (yy * WIDTH + xx) * 4;

                    frame[i] = 40;
                    frame[i + 1] = 120;
                    frame[i + 2] = 255;
                    frame[i + 3] = 255;
                }
            }
        }

        // Folder body
        let size = 48;

        for yy in y..y + size {
            for xx in x..x + size {
                if xx >= WIDTH || yy >= HEIGHT {
                    continue;
                }

                let i = (yy * WIDTH + xx) * 4;

                frame[i] = 245;
                frame[i + 1] = 210;
                frame[i + 2] = 40;
                frame[i + 3] = 255;
            }
        }

        // Folder tab
        for yy in y..y + 10 {
            for xx in x..x + 20 {
                if xx >= WIDTH || yy >= HEIGHT {
                    continue;
                }

                let i = (yy * WIDTH + xx) * 4;

                frame[i] = 255;
                frame[i + 1] = 225;
                frame[i + 2] = 80;
                frame[i + 3] = 255;
            }
        }

        // Icon label — centred below the icon
        let label     = &icon.name;
        let label_w   = TextRenderer::measure(label);
        let label_x   = if label_w < size {
            x + (size - label_w) / 2
        } else {
            x.saturating_sub((label_w - size) / 2)
        };
        let label_y   = y + size + 5;

        // Dim shadow offset (+1,+1) for readability on gradient bg
        TextRenderer::draw(frame, label_x + 1, label_y + 1, label, [0, 0, 0]);
        TextRenderer::draw(frame, label_x,     label_y,     label, [240, 240, 255]);
    }
}
pub struct DesktopRenderer;

use crate::display::Cursor;
use crate::display::WindowRenderer;
use crate::desktop_state::DesktopState;
use crate::display::IconRenderer;
use crate::display::context_menu::{ContextMenu, MENU_ITEM_H, MENU_ITEM_W, MENU_PAD_Y};
use crate::display::notification::{NotificationManager, TOAST_W, TOAST_H};
use crate::display::start_menu::{StartMenu, MENU_X, MENU_Y_FROM_BOTTOM, MENU_W, MENU_H, TILE_W, TILE_H, TILE_PAD_Y};
use crate::display::FontRenderer;
use crate::window_manager::{ResizeEdge, SnapZone};
use std::time::{SystemTime, UNIX_EPOCH};

const W: usize = 1280;
const H: usize = 720;
const TB: usize = 38;

impl DesktopRenderer {

    pub fn draw(frame: &mut [u8], desktop: &mut DesktopState) {
        let font = &desktop.font;
        Self::draw_background(frame, &mut desktop.wallpaper_cache);
        IconRenderer::draw(frame, &desktop.desktop_icons);
        Self::draw_taskbar(frame);
        Self::draw_start_button(frame, font);
        Self::draw_search_bar(frame, font);
        Self::draw_taskbar_buttons(frame, &desktop.window_manager.windows, font);
        Self::draw_clock(frame, font);
        if desktop.start_menu.visible {
            Self::draw_start_menu(frame, &desktop.start_menu, font);
        }
        if let Some(zone) = desktop.snap_preview {
            Self::draw_snap_preview(frame, zone);
        }
        WindowRenderer::draw(frame, &desktop.window_manager.windows, font);
        if desktop.alt_tab_visible {
            Self::draw_alt_tab(frame, &desktop.window_manager.windows, desktop.alt_tab_index, font);
        }
        if desktop.context_menu.visible {
            Self::draw_context_menu(frame, &desktop.context_menu, font);
        }
        Self::draw_toasts(frame, &desktop.notifications, font);
        Self::draw_cursor(frame, &desktop.cursor, desktop.hover_edge);
    }

    // ── Wallpaper ─────────────────────────────────────────────────────
    // Matches screenshot: deep purple-black top-left, concentrated blue bloom
    // top-right, vivid blue wash right-centre, tight magenta-purple bottom-left,
    // diagonal aurora band sweeping from bottom-left to upper-centre.
    fn draw_background(frame: &mut [u8], cache: &mut Vec<u8>) {
        if cache.len() == W*H*4 {
            frame[..W*H*4].copy_from_slice(&cache[..W*H*4]);
            return;
        }
        for y in 0..H {
            for x in 0..W {
                let fx = x as f32;
                let fy = y as f32;
                let tx = fx / W as f32;
                let ty = fy / H as f32;

                // Base gradient: near-black purple → dark blue
                let base_r = (28.0*(1.0-tx) + 10.0*tx) * (1.0-ty) + (55.0*(1.0-tx) + 12.0*tx) * ty;
                let base_g = (5.0*(1.0-tx)  + 18.0*tx) * (1.0-ty) + (3.0*(1.0-tx)  + 32.0*tx) * ty;
                let base_b = (75.0*(1.0-tx)  + 145.0*tx) * (1.0-ty) + (95.0*(1.0-tx) + 175.0*tx) * ty;

                // Bright blue-white bloom — top-right corner (the key feature)
                let tr_dx = fx - W as f32 * 0.91;
                let tr_dy = fy - H as f32 * 0.06;
                let tr_d  = (tr_dx*tr_dx + tr_dy*tr_dy).sqrt();
                let glow_tr = ((1.0 - tr_d / (W as f32 * 0.32)).max(0.0)).powf(1.5) * 110.0;

                // Large vivid blue wash — right centre
                let rc_dx = fx - W as f32 * 0.82;
                let rc_dy = fy - H as f32 * 0.52;
                let rc_d  = (rc_dx*rc_dx + rc_dy*rc_dy).sqrt();
                let glow_rc = ((1.0 - rc_d / (W as f32 * 0.48)).max(0.0)).powf(1.4) * 95.0;

                // Tight magenta-purple — bottom-left
                let bl_dx = fx - W as f32 * 0.03;
                let bl_dy = fy - H as f32 * 0.78;
                let bl_d  = (bl_dx*bl_dx + bl_dy*bl_dy).sqrt();
                let glow_bl = ((1.0 - bl_d / (W as f32 * 0.25)).max(0.0)).powf(2.0) * 80.0;

                // Aurora band — sweeps diagonally bottom-left to upper-centre
                // More contrast than before, narrower band
                let band_t = fx * 0.0022 - fy * 0.0048 + 0.18;
                let band   = (band_t.sin() * 0.5 + 0.5).powf(5.0) * 90.0;

                let r = (base_r + band*0.9  + glow_tr*0.18 + glow_bl*0.0).min(255.0) as u8;
                let g = (base_g + band*0.05 + glow_rc*0.3  + glow_bl*0.04).min(255.0) as u8;
                let b = (base_b + band*0.15 + glow_tr + glow_rc + glow_bl*0.65).min(255.0) as u8;

                let i = (y*W+x)*4;
                frame[i]=r; frame[i+1]=g; frame[i+2]=b; frame[i+3]=255;
            }
        }
        *cache = frame[..W*H*4].to_vec();
    }

    // ── Taskbar — near-invisible dark panel ───────────────────────────
    fn draw_taskbar(frame: &mut [u8]) {
        let y0 = H - TB;
        // 1px separator
        for x in 0..W {
            let i=(y0*W+x)*4;
            frame[i]=42; frame[i+1]=42; frame[i+2]=65; frame[i+3]=255;
        }
        // Body: rgba(8,8,14,0.90) — very dark, nearly opaque
        for y in (y0+1)..H {
            for x in 0..W {
                let i=(y*W+x)*4;
                let br=frame[i] as u16; let bg=frame[i+1] as u16; let bb=frame[i+2] as u16;
                let a:u16=230;
                frame[i]  =((8u16*a+br*(255-a))/255) as u8;
                frame[i+1]=((8u16*a+bg*(255-a))/255) as u8;
                frame[i+2]=((14u16*a+bb*(255-a))/255) as u8;
                frame[i+3]=255;
            }
        }
    }

    // ── Start button — lightning bolt ⚡ ──────────────────────────────
    fn draw_start_button(frame: &mut [u8], _font: &FontRenderer) {
        let cx = 20i32;
        let cy = (H - TB/2) as i32;

        // Lightning bolt pixel art — yellow-white at top, amber at bottom
        let pixels: &[(i32,i32)] = &[
            // Upper stroke
            (2,-8),(3,-8),(4,-8),(5,-8),
            (1,-7),(2,-7),(3,-7),(4,-7),
            (0,-6),(1,-6),(2,-6),(3,-6),
            (-1,-5),(0,-5),(1,-5),(2,-5),
            (-2,-4),(-1,-4),(0,-4),(1,-4),
            // Wide middle
            (-3,-3),(-2,-3),(-1,-3),(0,-3),(1,-3),(2,-3),(3,-3),
            // Lower stroke
            (-1,-2),(0,-2),(1,-2),(2,-2),
            (-2,-1),(-1,-1),(0,-1),(1,-1),
            (-3,0),(-2,0),(-1,0),(0,0),
            (-4,1),(-3,1),(-2,1),(-1,1),
            (-5,2),(-4,2),(-3,2),(-2,2),
            (-5,3),(-4,3),(-3,3),
            (-5,4),(-4,4),
        ];
        for &(dx,dy) in pixels {
            let px=cx+dx; let py=cy+dy;
            if px>=0&&py>=0&&(px as usize)<W&&(py as usize)<H {
                let i=(py as usize*W+px as usize)*4;
                let t=((dy+9) as f32/14.0).clamp(0.0,1.0);
                frame[i]  =(255.0-t*20.0) as u8;
                frame[i+1]=(228.0-t*50.0) as u8;
                frame[i+2]=(80.0 -t*60.0).max(15.0) as u8;
                frame[i+3]=255;
            }
        }
    }

    // ── Search bar ────────────────────────────────────────────────────
    fn draw_search_bar(frame: &mut [u8], font: &FontRenderer) {
        let bx=42usize; let by=H-TB+6; let bw=165usize; let bh=26usize;
        // Pill shape with radius = bh/2
        let r = bh/2;
        for y in by..(by+bh).min(H) {
            for x in bx..(bx+bw).min(W) {
                let lx=x-bx; let ly=y-by;
                // Left cap
                if lx < r {
                    let dx=r as i32-lx as i32; let dy=r as i32-ly as i32;
                    if dx*dx+dy*dy > (r*r) as i32 { continue; }
                }
                // Right cap
                if lx >= bw-r {
                    let rx=bw-lx-1;
                    let dx=r as i32-rx as i32; let dy=r as i32-ly as i32;
                    if dx*dx+dy*dy > (r*r) as i32 { continue; }
                }
                let i=(y*W+x)*4;
                frame[i]=28; frame[i+1]=28; frame[i+2]=44; frame[i+3]=255;
            }
        }
        font.draw(frame, bx+10, by+7, "Search", 11.0, [85,88,118]);
    }

    // ── Unified taskbar icon buttons ──────────────────────────────────
    fn draw_taskbar_buttons(frame:&mut[u8], windows:&[crate::window_manager::Window], font:&FontRenderer){
        struct App { name:&'static str, id:&'static str, col:(u8,u8,u8) }
        let apps=[
            App{name:"File Explorer",id:"explorer",  col:(240,180,40) },
            App{name:"Browser",      id:"browser",   col:(60,195,95)  },
            App{name:"Terminal",     id:"terminal",  col:(60,200,80)  },
            App{name:"TRiDOCS",      id:"tridocs",   col:(70,130,220) },
            App{name:"TRiSHEETS",    id:"trisheets", col:(50,180,100) },
            App{name:"TRiSLIDES",    id:"trislides", col:(220,100,60) },
        ];
        let btn=TB-4; let gap=4usize;
        let total=apps.len()*(btn+gap)-gap;
        let sx=(W-total)/2;

        for (pi,app) in apps.iter().enumerate() {
            let bx=sx+pi*(btn+gap); let by=H-TB+2;
            let open:Vec<&crate::window_manager::Window>=windows.iter().filter(|w|w.app==app.id).collect();
            let has_open=!open.is_empty();
            let is_active=open.iter().any(|w|w.active&&!w.minimized);
            let is_min=has_open&&open.iter().all(|w|w.minimized);

            // Hover/active background — very subtle
            if is_active {
                for y in by..(by+btn).min(H){ for x in bx..(bx+btn).min(W){
                    if x<W&&y<H{
                        let i=(y*W+x)*4;
                        frame[i]  =(frame[i]   as u16+22).min(255) as u8;
                        frame[i+1]=(frame[i+1] as u16+22).min(255) as u8;
                        frame[i+2]=(frame[i+2] as u16+32).min(255) as u8;
                        frame[i+3]=255;
                    }
                }}
            }

            // Icon — 18×18, centred
            let is=18usize; let ix=bx+(btn-is)/2; let iy=by+(btn-is)/2;
            let (ir,ig,ib)=app.col;
            Self::draw_app_icon(frame,app.id,ix,iy,is,ir,ig,ib,font);

            // Running indicator — dots under button
            if has_open {
                let n=open.len().min(3);
                let dw=if n==1{12}else if n==2{5}else{4};
                let dg=if n>1{3}else{0};
                let total_d=n*dw+(n-1)*dg;
                let dx=bx+(btn-total_d)/2;
                let dot_y=by+btn-3;
                for d in 0..n {
                    let ddx=dx+d*(dw+dg);
                    let (dr,dg2,db)=if is_active&&d==0{(75u8,145,255)}
                        else if is_min{(55,58,85)}
                        else{(110,115,150)};
                    for x in ddx..(ddx+dw).min(W){
                        for dy in 0..2usize{
                            let y=dot_y+dy;
                            if y<H&&x<W{let i=(y*W+x)*4;frame[i]=dr;frame[i+1]=dg2;frame[i+2]=db;frame[i+3]=255;}
                        }
                    }
                }
            }
            let _ = (font, app.name);
        }
    }

    fn draw_app_icon(frame:&mut[u8],id:&str,ix:usize,iy:usize,is:usize,ir:u8,ig:u8,ib:u8,font:&FontRenderer){
        match id {
            "explorer" => {
                // Folder body
                for y in (iy+4)..(iy+is).min(H){ for x in ix..(ix+is).min(W){
                    if x<W&&y<H{let i=(y*W+x)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
                }}
                // Tab (lighter)
                for y in iy..(iy+5).min(H){ for x in ix..(ix+8).min(W){
                    if x<W&&y<H{let i=(y*W+x)*4;
                    frame[i]=(ir as u16+20).min(255) as u8;
                    frame[i+1]=(ig as u16+20).min(255) as u8;
                    frame[i+2]=ib;frame[i+3]=255;}
                }}
            }
            "browser" => {
                // Clean globe: ring + cross
                let r=(is/2) as i32; let cx=(ix+is/2) as i32; let cy=(iy+is/2) as i32;
                for dy in -r..=r{ for dx in -r..=r{
                    let d2=dx*dx+dy*dy;
                    let px=(cx+dx) as usize; let py=(cy+dy) as usize;
                    if px<W&&py<H{
                        let ring=d2<=r*r&&d2>=(r-2)*(r-2);
                        let cross=d2<r*r&&(dx==0||dy==0);
                        if ring||cross{let i=(py*W+px)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
                    }
                }}
            }
            "terminal" => {
                // Dark bg
                for y in iy..(iy+is).min(H){ for x in ix..(ix+is).min(W){
                    if x<W&&y<H{let i=(y*W+x)*4;frame[i]=20;frame[i+1]=20;frame[i+2]=20;frame[i+3]=255;}
                }}
                // > chevron
                let cx=(ix+4) as i32; let cy=(iy+is/2) as i32;
                for k in 0..5i32{
                    for s in [-1i32,1]{
                        let px=(cx+k) as usize; let py=(cy+s*k) as usize;
                        if px<W&&py<H{let i=(py*W+px)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
                    }
                }
                // _ underline
                for x in (ix+10)..(ix+16).min(W){
                    let py=iy+is-5;
                    if py<H{let i=(py*W+x)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
                    if py+1<H{let i=((py+1)*W+x)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
                }
            }
            _ => {
                // Rounded coloured square
                let r2=3usize;
                for y in iy..(iy+is).min(H){ for x in ix..(ix+is).min(W){
                    let lx=x-ix;let ly=y-iy;let rx=is-lx-1;let ry=is-ly-1;
                    if lx<r2&&ly<r2{let dx=r2-lx;let dy=r2-ly;if dx*dx+dy*dy>r2*r2{continue;}}
                    if rx<r2&&ly<r2{let dx=r2-rx;let dy=r2-ly;if dx*dx+dy*dy>r2*r2{continue;}}
                    if lx<r2&&ry<r2{let dx=r2-lx;let dy=r2-ry;if dx*dx+dy*dy>r2*r2{continue;}}
                    if rx<r2&&ry<r2{let dx=r2-rx;let dy=r2-ry;if dx*dx+dy*dy>r2*r2{continue;}}
                    if x<W&&y<H{let i=(y*W+x)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
                }}
                let first=id.chars().next().unwrap_or('?').to_uppercase().to_string();
                font.draw(frame,ix+4,iy+2,&first,10.0,[240,242,255]);
            }
        }
    }

    // ── Clock ─────────────────────────────────────────────────────────
    fn draw_clock(frame:&mut[u8],font:&FontRenderer){
        let secs=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let local=secs+19800; // IST
        let mins=local/60; let h24=(mins/60)%24; let min=mins%60;
        let h12=match h24%12{0=>12,h=>h};
        let ampm=if h24<12{"AM"}else{"PM"};
        let ts=format!("{:02}:{:02}",h12,min);
        let tw=font.measure(&ts,12.0) as usize;
        let aw=font.measure(ampm,10.0) as usize;
        let rx=W-8-tw.max(aw);
        font.draw(frame,rx,H-TB+6,&ts,12.0,[192,198,230]);
        font.draw(frame,rx+(tw.saturating_sub(aw))/2,H-TB+20,ampm,10.0,[115,120,155]);
    }

    // ── Start menu — Windows 11 style: centred, wide, search + tile grid ─
    fn draw_start_menu(frame:&mut[u8], menu:&StartMenu, font:&FontRenderer){
        let mt = H - MENU_Y_FROM_BOTTOM;
        let r  = 12usize;

        // ── Panel background — frosted dark glass ───────────────────
        for y in mt..(mt+MENU_H).min(H) {
            for x in MENU_X..(MENU_X+MENU_W).min(W) {
                let lx=x-MENU_X; let ly=y-mt;
                let rx2=MENU_W-lx-1; let ry=MENU_H-ly-1;
                // Rounded corners (all four)
                if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
                if rx2<r&&ly<r{let dx=r-rx2;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
                if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
                if rx2<r&&ry<r{let dx=r-rx2;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}

                let i=(y*W+x)*4;
                let br=frame[i] as u16; let bg=frame[i+1] as u16; let bb=frame[i+2] as u16;
                // 88% opaque dark — slightly lighter than taskbar
                let a:u16=226;
                frame[i]  =((24u16*a+br*(255-a))/255) as u8;
                frame[i+1]=((24u16*a+bg*(255-a))/255) as u8;
                frame[i+2]=((38u16*a+bb*(255-a))/255) as u8;
                frame[i+3]=255;
            }
        }

        // ── Border — 1px all around ──────────────────────────────────
        for y in mt..(mt+MENU_H).min(H) {
            for x in MENU_X..(MENU_X+MENU_W).min(W) {
                let lx=x-MENU_X; let ly=y-mt; let rx2=MENU_W-lx-1; let ry=MENU_H-ly-1;
                if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
                if rx2<r&&ly<r{let dx=r-rx2;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
                if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
                if rx2<r&&ry<r{let dx=r-rx2;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
                if lx>0&&rx2>0&&ly>0&&ry>0{continue;}
                let i=(y*W+x)*4;
                frame[i]=52; frame[i+1]=52; frame[i+2]=88; frame[i+3]=255;
            }
        }

        // ── User profile strip at top ────────────────────────────────
        // Avatar circle
        let av_x=(MENU_X+22) as i32; let av_y=(mt+20) as i32; let av_r=16i32;
        for dy in -av_r..=av_r { for dx in -av_r..=av_r {
            if dx*dx+dy*dy<=av_r*av_r {
                let px=(av_x+dx) as usize; let py=(av_y+dy) as usize;
                if px<W&&py<H {
                    let i=(py*W+px)*4;
                    // Purple-blue gradient avatar
                    let t=(dy+av_r) as f32/(av_r*2) as f32;
                    frame[i]  =(80.0-t*20.0) as u8;
                    frame[i+1]=(60.0-t*10.0) as u8;
                    frame[i+2]=(180.0+t*20.0) as u8;
                    frame[i+3]=255;
                }
            }
        }}
        font.draw(frame, av_x as usize-4, mt+12, "A", 14.0, [220,225,255]);
        font.draw(frame, MENU_X+46, mt+14, "Adithya", 13.0, [218,222,255]);
        font.draw(frame, MENU_X+46, mt+29, "Trimangees OS", 10.5, [110,115,155]);

        // ── Search bar ───────────────────────────────────────────────
        let sb_x=MENU_X+20; let sb_y=mt+54; let sb_w=MENU_W-40; let sb_h=32usize; let sb_r=8usize;
        for y in sb_y..(sb_y+sb_h).min(H) {
            for x in sb_x..(sb_x+sb_w).min(W) {
                let lx=x-sb_x; let ly=y-sb_y; let rx2=sb_w-lx-1; let ry=sb_h-ly-1;
                if lx<sb_r&&ly<sb_r{let dx=sb_r-lx;let dy=sb_r-ly;if dx*dx+dy*dy>sb_r*sb_r{continue;}}
                if rx2<sb_r&&ly<sb_r{let dx=sb_r-rx2;let dy=sb_r-ly;if dx*dx+dy*dy>sb_r*sb_r{continue;}}
                if lx<sb_r&&ry<sb_r{let dx=sb_r-lx;let dy=sb_r-ry;if dx*dx+dy*dy>sb_r*sb_r{continue;}}
                if rx2<sb_r&&ry<sb_r{let dx=sb_r-rx2;let dy=sb_r-ry;if dx*dx+dy*dy>sb_r*sb_r{continue;}}
                let i=(y*W+x)*4;
                // Border pixel
                if lx==0||rx2==0||ly==0||ry==0 {
                    frame[i]=62;frame[i+1]=62;frame[i+2]=105;frame[i+3]=255;
                } else {
                    frame[i]=18;frame[i+1]=18;frame[i+2]=30;frame[i+3]=255;
                }
            }
        }
        // Search icon — simple magnifier dot
        let si_x=sb_x+12; let si_y=sb_y+sb_h/2-4;
        for dy in -4i32..=4 { for dx in -4i32..=4 {
            let d2=dx*dx+dy*dy;
            if d2<=16&&d2>=9 {
                let px=(si_x as i32+dx) as usize; let py=(si_y as i32+dy) as usize;
                if px<W&&py<H{let i=(py*W+px)*4;frame[i]=85;frame[i+1]=88;frame[i+2]=125;frame[i+3]=255;}
            }
        }}
        // Handle line
        for d in 4..8i32 {
            let px=(si_x as i32+d) as usize; let py=(si_y as i32+d) as usize;
            if px<W&&py<H{let i=(py*W+px)*4;frame[i]=85;frame[i+1]=88;frame[i+2]=125;frame[i+3]=255;}
        }
        font.draw(frame, sb_x+28, sb_y+10, "Search apps and files", 11.5, [72,75,108]);

        // ── "Pinned" section label ────────────────────────────────────
        font.draw(frame, MENU_X+20, mt+TILE_PAD_Y-18, "Pinned", 11.0, [115,120,162]);

        // ── App tile grid ────────────────────────────────────────────
        for (i, entry) in menu.entries.iter().enumerate() {
            let (tx, ty) = StartMenu::tile_pos(i, mt as i32);
            let tx=tx as usize; let ty=ty as usize;
            let hov = menu.hovered == Some(i);
            let tr2=8usize;

            // Tile background
            let (tbr,tbg,tbb) = if hov {(50,52,80)} else {(32,34,56)};
            for yy in ty..(ty+TILE_H).min(H) {
                for xx in tx..(tx+TILE_W).min(W) {
                    let lx=xx-tx;let ly=yy-ty;let rx3=TILE_W-lx-1;let ry=TILE_H-ly-1;
                    if lx<tr2&&ly<tr2{let dx=tr2-lx;let dy=tr2-ly;if dx*dx+dy*dy>tr2*tr2{continue;}}
                    if rx3<tr2&&ly<tr2{let dx=tr2-rx3;let dy=tr2-ly;if dx*dx+dy*dy>tr2*tr2{continue;}}
                    if lx<tr2&&ry<tr2{let dx=tr2-lx;let dy=tr2-ry;if dx*dx+dy*dy>tr2*tr2{continue;}}
                    if rx3<tr2&&ry<tr2{let dx=tr2-rx3;let dy=tr2-ry;if dx*dx+dy*dy>tr2*tr2{continue;}}
                    if xx<W&&yy<H{let idx=(yy*W+xx)*4;frame[idx]=tbr;frame[idx+1]=tbg;frame[idx+2]=tbb;frame[idx+3]=255;}
                }
            }

            // Icon square — rounded, larger, centred in top 2/3 of tile
            let is=36usize; let ix=tx+(TILE_W-is)/2; let iy=ty+8; let ir2=6usize;
            let [ic_r,ic_g,ic_b]=entry.icon_color;
            for yy in iy..(iy+is).min(H) {
                for xx in ix..(ix+is).min(W) {
                    let lx=xx-ix;let ly=yy-iy;let rx3=is-lx-1;let ry=is-ly-1;
                    if lx<ir2&&ly<ir2{let dx=ir2-lx;let dy=ir2-ly;if dx*dx+dy*dy>ir2*ir2{continue;}}
                    if rx3<ir2&&ly<ir2{let dx=ir2-rx3;let dy=ir2-ly;if dx*dx+dy*dy>ir2*ir2{continue;}}
                    if lx<ir2&&ry<ir2{let dx=ir2-lx;let dy=ir2-ry;if dx*dx+dy*dy>ir2*ir2{continue;}}
                    if rx3<ir2&&ry<ir2{let dx=ir2-rx3;let dy=ir2-ry;if dx*dx+dy*dy>ir2*ir2{continue;}}
                    // Slight gradient: lighter top, darker bottom
                    let t=ly as f32/is as f32;
                    let r2=(ic_r as f32*(1.0-t*0.25)) as u8;
                    let g2=(ic_g as f32*(1.0-t*0.25)) as u8;
                    let b2=(ic_b as f32*(1.0-t*0.25)) as u8;
                    if xx<W&&yy<H{let idx=(yy*W+xx)*4;frame[idx]=r2;frame[idx+1]=g2;frame[idx+2]=b2;frame[idx+3]=255;}
                }
            }

            // Icon letter — centred in the square
            let letter = entry.icon_char.to_string();
            let lw = font.measure(&letter, 14.0) as usize;
            let lh = 14usize;
            font.draw(frame, ix+(is.saturating_sub(lw))/2, iy+(is-lh)/2, &letter, 14.0, [240,244,255]);

            // App name — centred in bottom 1/3 of tile
            let name = font.truncate(&entry.name, 10.5, (TILE_W-6) as f32);
            let nw = font.measure(&name, 10.5) as usize;
            font.draw(frame, tx+(TILE_W.saturating_sub(nw))/2, iy+is+6, &name, 10.5, [185,190,228]);
        }

        // ── Separator before power ───────────────────────────────────
        let sep_y = mt+MENU_H-50;
        for x in (MENU_X+16)..(MENU_X+MENU_W-16).min(W) {
            if sep_y<H&&x<W{let i=(sep_y*W+x)*4;frame[i]=48;frame[i+1]=48;frame[i+2]=80;frame[i+3]=255;}
        }

        // ── Power buttons ────────────────────────────────────────────
        let pw=88usize; let ph=28usize; let pg=10usize;
        let pt=3*pw+2*pg;
        let psx=MENU_X+(MENU_W-pt)/2;
        let pby=mt+MENU_H-42;

        let labels=["⏾  Sleep","↺  Restart","⏻  Shutdown"];
        for (bi,lbl) in labels.iter().enumerate() {
            let pbx=psx+bi*(pw+pg);
            let hov=menu.power_hovered==Some(bi);
            let pr2=6usize;
            let (pbr,pbg,pbb):(u8,u8,u8)=match bi {
                2=>if hov{(185,45,45)}else{(110,25,25)},
                1=>if hov{(45,85,175)}else{(28,58,128)},
                _=>if hov{(42,45,70)}else{(28,30,50)},
            };
            for y in pby..(pby+ph).min(H) {
                for x in pbx..(pbx+pw).min(W) {
                    let lx=x-pbx;let ly=y-pby;let rx2=pw-lx-1;let ry=ph-ly-1;
                    if lx<pr2&&ly<pr2{let dx=pr2-lx;let dy=pr2-ly;if dx*dx+dy*dy>pr2*pr2{continue;}}
                    if rx2<pr2&&ly<pr2{let dx=pr2-rx2;let dy=pr2-ly;if dx*dx+dy*dy>pr2*pr2{continue;}}
                    if lx<pr2&&ry<pr2{let dx=pr2-lx;let dy=pr2-ry;if dx*dx+dy*dy>pr2*pr2{continue;}}
                    if rx2<pr2&&ry<pr2{let dx=pr2-rx2;let dy=pr2-ry;if dx*dx+dy*dy>pr2*pr2{continue;}}
                    if x<W&&y<H{let i=(y*W+x)*4;frame[i]=pbr;frame[i+1]=pbg;frame[i+2]=pbb;frame[i+3]=255;}
                }
            }
            // Border
            for x in pbx..(pbx+pw).min(W) {
                if pby<H&&x<W{let i=(pby*W+x)*4;frame[i]=(pbr as u16+35).min(255) as u8;frame[i+1]=(pbg as u16+35).min(255) as u8;frame[i+2]=(pbb as u16+45).min(255) as u8;frame[i+3]=255;}
            }
            let lw=font.measure(lbl,11.0) as usize;
            font.draw(frame,pbx+(pw.saturating_sub(lw))/2,pby+(ph-11)/2,lbl,11.0,[210,215,245]);
        }
    }

    // ── Context menu ──────────────────────────────────────────────────
    // ── Context menu ──────────────────────────────────────────────────
    fn draw_context_menu(frame:&mut[u8],menu:&ContextMenu,font:&FontRenderer){
        let mx=menu.x as usize; let my=menu.y as usize;
        let mw=MENU_ITEM_W; let mh=menu.height(); let r=7usize;
        for y in my..(my+mh).min(H){ for x in mx..(mx+mw).min(W){
            let lx=x-mx;let ly=y-my;let rx2=mw-lx-1;let ry=mh-ly-1;
            if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
            if rx2<r&&ly<r{let dx=r-rx2;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
            if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
            if rx2<r&&ry<r{let dx=r-rx2;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
            let i=(y*W+x)*4;
            frame[i]=22;frame[i+1]=22;frame[i+2]=36;frame[i+3]=255;
        }}
        for x in mx..(mx+mw).min(W){
            let i=(my*W+x)*4;frame[i]=58;frame[i+1]=58;frame[i+2]=95;frame[i+3]=255;
            let ib=((my+mh-1).min(H-1)*W+x)*4;frame[ib]=58;frame[ib+1]=58;frame[ib+2]=95;frame[ib+3]=255;
        }
        let mut cy=menu.y+MENU_PAD_Y as i32;
        for (i,item) in menu.items.iter().enumerate(){
            if item.separator{
                let sy=(cy+4) as usize;
                for x in (mx+8)..(mx+mw-8).min(W){ if sy<H{ let idx=(sy*W+x)*4;frame[idx]=48;frame[idx+1]=48;frame[idx+2]=78;frame[idx+3]=255; }}
                cy+=9;
            } else {
                if menu.hovered==Some(i){
                    for hy in cy as usize..(cy as usize+MENU_ITEM_H).min(H){
                        for hx in (mx+4)..(mx+mw-4).min(W){ let idx=(hy*W+hx)*4;frame[idx]=40;frame[idx+1]=40;frame[idx+2]=68;frame[idx+3]=255; }
                    }
                }
                font.draw(frame,mx+12,cy as usize+(MENU_ITEM_H-12)/2,&item.label,12.0,[208,212,240]);
                cy+=MENU_ITEM_H as i32;
            }
        }
    }

    // ── Toasts ────────────────────────────────────────────────────────
    fn draw_toasts(frame:&mut[u8],mgr:&NotificationManager,font:&FontRenderer){
        for (i,toast) in mgr.toasts.iter().enumerate(){
            let (tx,ty)=NotificationManager::toast_pos(i,W,H);
            let tx=tx as usize; let ty=ty as usize; let r=8usize;
            let age=toast.created.elapsed().as_secs_f32()/3.0;
            let af=if age>0.83{1.0-(age-0.83)/0.17}else{1.0};
            let a=(af*218.0) as u16;
            for y in ty..(ty+TOAST_H).min(H){ for x in tx..(tx+TOAST_W).min(W){
                let lx=x-tx;let ly=y-ty;let rx2=TOAST_W-lx-1;let ry=TOAST_H-ly-1;
                if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
                if rx2<r&&ly<r{let dx=r-rx2;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
                if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
                if rx2<r&&ry<r{let dx=r-rx2;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
                let idx=(y*W+x)*4;
                let br=frame[idx] as u16;let bg=frame[idx+1] as u16;let bb=frame[idx+2] as u16;
                frame[idx]  =((22u16*a+br*(255-a))/255) as u8;
                frame[idx+1]=((24u16*a+bg*(255-a))/255) as u8;
                frame[idx+2]=((40u16*a+bb*(255-a))/255) as u8;
                frame[idx+3]=255;
            }}
            for y in (ty+4)..(ty+TOAST_H-4).min(H){ for x in tx..(tx+4).min(W){
                let i=(y*W+x)*4;frame[i]=82;frame[i+1]=155;frame[i+2]=255;frame[i+3]=255;
            }}
            font.draw(frame,tx+12,ty+10,&toast.title,12.0,[220,224,255]);
            let bs=font.truncate(&toast.body,11.0,(TOAST_W-20) as f32);
            font.draw(frame,tx+12,ty+24,&bs,11.0,[140,145,185]);
            let bw=((1.0-age)*(TOAST_W-8) as f32) as usize;
            for x in (tx+4)..(tx+4+bw).min(tx+TOAST_W-4).min(W){
                let y=ty+TOAST_H-3;if y<H{let i=(y*W+x)*4;frame[i]=68;frame[i+1]=128;frame[i+2]=208;frame[i+3]=255;}
            }
        }
    }

    // ── Alt+Tab ───────────────────────────────────────────────────────
    fn draw_alt_tab(frame:&mut[u8],windows:&[crate::window_manager::Window],sel:usize,font:&FontRenderer){
        let vis:Vec<_>=windows.iter().filter(|w|!w.minimized).collect();
        if vis.is_empty(){return;}
        let tw=148usize;let th=92usize;let pad=16usize;let gap=10usize;let lh=20usize;
        let pw=pad*2+vis.len()*(tw+gap)-gap; let ph=pad*2+th+lh;
        let px=(W.saturating_sub(pw))/2; let py=(H.saturating_sub(ph))/2; let r=12usize;
        for y in py..(py+ph).min(H){ for x in px..(px+pw).min(W){
            let lx=x-px;let ly=y-py;let rx2=pw-lx-1;let ry=ph-ly-1;
            if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
            if rx2<r&&ly<r{let dx=r-rx2;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
            if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
            if rx2<r&&ry<r{let dx=r-rx2;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
            let i=(y*W+x)*4;
            let br=frame[i] as u16;let bg=frame[i+1] as u16;let bb=frame[i+2] as u16;
            let a:u16=238;
            frame[i]  =((16u16*a+br*(255-a))/255) as u8;
            frame[i+1]=((16u16*a+bg*(255-a))/255) as u8;
            frame[i+2]=((26u16*a+bb*(255-a))/255) as u8;
            frame[i+3]=255;
        }}
        for x in px..(px+pw).min(W){
            let i=(py*W+x)*4;frame[i]=62;frame[i+1]=62;frame[i+2]=100;frame[i+3]=255;
        }
        for (i,win) in vis.iter().enumerate(){
            let tx=px+pad+i*(tw+gap); let ty=py+pad;
            let sel_i=sel%vis.len();
            if i==sel_i{
                for y in ty.saturating_sub(4)..(ty+th+4).min(H){
                    for x in tx.saturating_sub(4)..(tx+tw+4).min(W){
                        if x<W&&y<H{let i=(y*W+x)*4;frame[i]=42;frame[i+1]=108;frame[i+2]=218;frame[i+3]=255;}
                    }
                }
            }
            // Thumbnail bg
            for y in ty..(ty+th).min(H){ for x in tx..(tx+tw).min(W){
                if x<W&&y<H{let idx=(y*W+x)*4;frame[idx]=14;frame[idx+1]=14;frame[idx+2]=22;frame[idx+3]=255;}
            }}
            // Mini title bar
            for y in ty..(ty+18).min(H){ for x in tx..(tx+tw).min(W){
                if x<W&&y<H{let t=(y-ty) as f32/18.0; let idx=(y*W+x)*4;
                    frame[idx]  =lerp_f(74.0,56.0,t) as u8;
                    frame[idx+1]=lerp_f(138.0,114.0,t) as u8;
                    frame[idx+2]=lerp_f(240.0,216.0,t) as u8;
                    frame[idx+3]=255;
                }
            }}
            let ttw=font.measure(&win.title,10.0) as usize;
            font.draw(frame,tx+(tw.saturating_sub(ttw))/2,ty+4,&win.title,10.0,[228,232,255]);
            let col=if i==sel_i{[255u8,255,255]}else{[150,155,192]};
            let lbl=font.truncate(&win.title,11.0,tw as f32);
            let lw=font.measure(&lbl,11.0) as usize;
            font.draw(frame,tx+(tw.saturating_sub(lw))/2,ty+th+5,&lbl,11.0,col);
        }
    }

    // ── Snap preview ──────────────────────────────────────────────────
    fn draw_snap_preview(frame:&mut[u8],zone:SnapZone){
        let (x,y,w,h)=zone.geometry();
        let x0=x.max(0) as usize; let y0=y.max(0) as usize;
        let x1=(x+w as i32).min(W as i32) as usize; let y1=(y+h as i32).min(682) as usize;
        let r=8usize;
        for py in y0..y1{ for px in x0..x1{
            let lx=px-x0;let ly=py-y0;let rx2=x1-px-1;let ry=y1-py-1;
            if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
            if rx2<r&&ly<r{let dx=r-rx2;let dy=r-ly;if dx*dx+dy*dy>r*r{continue;}}
            if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
            if rx2<r&&ry<r{let dx=r-rx2;let dy=r-ry;if dx*dx+dy*dy>r*r{continue;}}
            let i=(py*W+px)*4;
            let br=frame[i] as u16; let bg=frame[i+1] as u16; let bb=frame[i+2] as u16;
            if lx<2||rx2<2||ly<2||ry<2{
                frame[i]=72;frame[i+1]=150;frame[i+2]=255;frame[i+3]=255;
            } else {
                let a:u16=65;
                frame[i]  =((35u16*a+br*(255-a))/255) as u8;
                frame[i+1]=((92u16*a+bg*(255-a))/255) as u8;
                frame[i+2]=((210u16*a+bb*(255-a))/255) as u8;
                frame[i+3]=255;
            }
        }}
    }

    // ── Cursor ────────────────────────────────────────────────────────
    fn draw_cursor(frame:&mut[u8],cursor:&Cursor,edge:Option<ResizeEdge>){
        let cx=cursor.x; let cy=cursor.y;
        match edge {
            Some(ResizeEdge::Left)|Some(ResizeEdge::Right)=>{
                for dx in -7i32..=7{ Self::dot(frame,cx+dx,cy,[255,255,255]); }
                for d in 1..=4i32{ for dy in -d..=d{
                    Self::dot(frame,cx-7+d,cy+dy,[255,255,255]);
                    Self::dot(frame,cx+7-d,cy+dy,[255,255,255]);
                }}
            }
            Some(ResizeEdge::Top)|Some(ResizeEdge::Bottom)=>{
                for dy in -7i32..=7{ Self::dot(frame,cx,cy+dy,[255,255,255]); }
                for d in 1..=4i32{ for dx in -d..=d{
                    Self::dot(frame,cx+dx,cy-7+d,[255,255,255]);
                    Self::dot(frame,cx+dx,cy+7-d,[255,255,255]);
                }}
            }
            Some(_)=>{
                for d in -5i32..=5{
                    Self::dot(frame,cx+d,cy+d,[255,255,255]);
                    Self::dot(frame,cx+d,cy-d,[255,255,255]);
                }
            }
            None=>{
                // Arrow with dark outline for visibility
                for dy in 0..13i32{ for dx in 0..13i32{
                    if dx+dy<13{
                        let c=if dx+dy>=11{[0u8,0,0]}else{[255,255,255]};
                        Self::dot(frame,cx+dx,cy+dy,c);
                    }
                }}
            }
        }
    }

    fn dot(frame:&mut[u8],x:i32,y:i32,c:[u8;3]){
        if x<0||y<0||(x as usize)>=W||(y as usize)>=H{return;}
        let i=(y as usize*W+x as usize)*4;
        frame[i]=c[0];frame[i+1]=c[1];frame[i+2]=c[2];frame[i+3]=255;
    }
}

fn lerp_f(a:f32,b:f32,t:f32)->f32{ a*(1.0-t)+b*t }
fn lerp(a:u8,b:u8,t:f32)->u8{ lerp_f(a as f32,b as f32,t) as u8 }
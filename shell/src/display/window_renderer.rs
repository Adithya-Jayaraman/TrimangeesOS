use crate::window_manager::{Window, FileEntry, SettingsState};
use crate::display::FontRenderer;

pub struct WindowRenderer;

const SW: usize = 1280;
const SH: usize = 720;
const TITLE_H: usize = 32;
const RADIUS:  usize = 8;

impl WindowRenderer {
    pub fn draw(frame: &mut [u8], windows: &[Window], font: &FontRenderer) {
        for w in windows {
            if w.minimized { continue; }
            Self::draw_window(frame, w, font);
        }
    }

    fn draw_window(frame: &mut [u8], win: &Window, font: &FontRenderer) {
        let x0 = win.x.max(0) as usize;
        let y0 = win.y.max(0) as usize;
        let x1 = (win.x + win.width  as i32).clamp(0, SW as i32) as usize;
        let y1 = (win.y + win.height as i32).clamp(0, SH as i32) as usize;
        if x1 <= x0 + 4 || y1 <= y0 + 4 { return; }

        let bcy      = y0 + TITLE_H / 2;
        let close_x  = x1.saturating_sub(18);
        let max_x    = close_x.saturating_sub(26);
        let min_x    = max_x.saturating_sub(26);

        // Window body
        for y in y0..y1 {
            for x in x0..x1 {
                let lx=x-x0;let ly=y-y0;let rx=x1-x-1;let ry=y1-y-1;
                if !Self::rr(lx,ly,rx,ry,RADIUS){continue;}
                let i=(y*SW+x)*4;
                if ly < TITLE_H {
                    let t=ly as f32/TITLE_H as f32;
                    if win.active {
                        frame[i]  =lerp(74,56,t); frame[i+1]=lerp(138,114,t); frame[i+2]=lerp(240,216,t);
                    } else {
                        frame[i]  =lerp(55,48,t); frame[i+1]=lerp(58,52,t);  frame[i+2]=lerp(82,72,t);
                    }
                } else {
                    frame[i]=16; frame[i+1]=16; frame[i+2]=24;
                }
                frame[i+3]=255;
            }
        }

        // Border
        for y in y0..y1 { for x in x0..x1 {
            let lx=x-x0;let ly=y-y0;let rx=x1-x-1;let ry=y1-y-1;
            if !Self::rr(lx,ly,rx,ry,RADIUS){continue;}
            if lx>0&&rx>0&&ly>0&&ry>0{continue;}
            let i=(y*SW+x)*4;
            if ly==0 {
                let c=if win.active{[92u8,158,248]}else{[62,66,98]};
                frame[i]=c[0];frame[i+1]=c[1];frame[i+2]=c[2];frame[i+3]=255;
            } else {
                frame[i]=32;frame[i+1]=35;frame[i+2]=58;frame[i+3]=255;
            }
        }}

        // Glass sheen
        if y0<SH { for x in (x0+RADIUS)..(x1.saturating_sub(RADIUS)) {
            if x>=SW{break;}
            let i=(y0*SW+x)*4;
            frame[i]  =((frame[i]   as u16*205+255*50)/255) as u8;
            frame[i+1]=((frame[i+1] as u16*205+255*50)/255) as u8;
            frame[i+2]=((frame[i+2] as u16*205+255*50)/255) as u8;
        }}

        // Title/body separator
        let sep=y0+TITLE_H;
        if sep<SH { for x in x0..x1 {
            if x<SW{let i=(sep*SW+x)*4;frame[i]=22;frame[i+1]=26;frame[i+2]=48;frame[i+3]=255;}
        }}

        // App icon circle
        let (cr,cg,cb):(u8,u8,u8)=match win.app.as_str(){
            "terminal"  =>(60,200,80),  "explorer" =>(240,180,40),
            "browser"   =>(60,150,255), "tridocs"  =>(70,130,220),
            "trisheets" =>(50,180,100), "trislides"=>(220,100,60),
            "tridraw"   =>(180,80,220), "settings" =>(130,135,175),
            _ =>(160,160,200),
        };
        let ic_x=(x0+12) as i32; let ic_y=bcy as i32; let ic_r=6i32;
        for dy in -ic_r..=ic_r { for dx in -ic_r..=ic_r {
            if dx*dx+dy*dy<=ic_r*ic_r {
                let px=(ic_x+dx) as usize; let py=(ic_y+dy) as usize;
                if px<SW&&py<SH{let i=(py*SW+px)*4;frame[i]=cr;frame[i+1]=cg;frame[i+2]=cb;frame[i+3]=255;}
            }
        }}

        // Title text
        let title_x=x0+27; let title_y=y0+8;
        let max_w=min_x.saturating_sub(title_x+8) as f32;
        let title=font.truncate(&win.title,13.0,max_w);
        let tcol=if win.active{[238u8,242,255]}else{[145,150,182]};
        font.draw(frame,title_x,title_y,&title,13.0,tcol);

        // Minimize line
        let lc=[185u8,190,218];
        for x in (min_x.saturating_sub(5))..(min_x+6) {
            if x<SW{
                if bcy<SH{let i=(bcy*SW+x)*4;frame[i]=lc[0];frame[i+1]=lc[1];frame[i+2]=lc[2];frame[i+3]=255;}
                if bcy+1<SH{let i=((bcy+1)*SW+x)*4;frame[i]=lc[0];frame[i+1]=lc[1];frame[i+2]=lc[2];frame[i+3]=255;}
            }
        }

        // Maximize square
        let sq=7i32;
        for d in -sq..=sq { for &e in &[-sq,sq] {
            let px=(max_x as i32+d) as usize; let py=(bcy as i32+e) as usize;
            if px<SW&&py<SH{let i=(py*SW+px)*4;frame[i]=lc[0];frame[i+1]=lc[1];frame[i+2]=lc[2];frame[i+3]=255;}
            let px2=(max_x as i32+e) as usize; let py2=(bcy as i32+d) as usize;
            if px2<SW&&py2<SH{let i=(py2*SW+px2)*4;frame[i]=lc[0];frame[i+1]=lc[1];frame[i+2]=lc[2];frame[i+3]=255;}
        }}

        // Close circle + X
        let cr2=8i32;
        for dy in -cr2..=cr2 { for dx in -cr2..=cr2 {
            if dx*dx+dy*dy<=cr2*cr2 {
                let px=(close_x as i32+dx) as usize; let py=(bcy as i32+dy) as usize;
                if px<SW&&py<SH{let i=(py*SW+px)*4;frame[i]=218;frame[i+1]=58;frame[i+2]=58;frame[i+3]=255;}
            }
        }}
        for d in -4i32..=4 { for thick in 0..2i32 {
            let px=(close_x as i32+d+thick) as usize;
            let py1=(bcy as i32+d) as usize; let py2=(bcy as i32-d) as usize;
            if px<SW{
                if py1<SH{let i=(py1*SW+px)*4;frame[i]=255;frame[i+1]=255;frame[i+2]=255;frame[i+3]=255;}
                if py2<SH{let i=(py2*SW+px)*4;frame[i]=255;frame[i+1]=255;frame[i+2]=255;frame[i+3]=255;}
            }
        }}

        // App content
        let cy0=(y0+TITLE_H+1).min(SH);
        match win.app.as_str() {
            "terminal" => Self::draw_terminal_content(frame,x0,cy0,x1,y1,font,
                &win.output_lines,&win.input_buffer,win.cursor_blink,win.scroll_offset),
            "explorer" => {
                if let Some(exp) = &win.explorer {
                    Self::draw_explorer(frame,x0,cy0,x1,y1,font,exp);
                }
            }
            "settings" => {
                if let Some(s) = &win.settings {
                    Self::draw_settings(frame,x0,cy0,x1,y1,font,s);
                }
            }
            _ => Self::draw_placeholder(frame,x0,cy0,x1,y1,font,&win.app),
        }
    }

    fn rr(lx:usize,ly:usize,rx:usize,ry:usize,r:usize)->bool{
        if lx<r&&ly<r{let dx=r-lx;let dy=r-ly;if dx*dx+dy*dy>r*r{return false;}}
        if rx<r&&ly<r{let dx=r-rx;let dy=r-ly;if dx*dx+dy*dy>r*r{return false;}}
        if lx<r&&ry<r{let dx=r-lx;let dy=r-ry;if dx*dx+dy*dy>r*r{return false;}}
        if rx<r&&ry<r{let dx=r-rx;let dy=r-ry;if dx*dx+dy*dy>r*r{return false;}}
        true
    }

    // ── Terminal ──────────────────────────────────────────────────────
    pub fn draw_terminal_content(
        frame:&mut[u8],x0:usize,y0:usize,x1:usize,y1:usize,
        font:&FontRenderer,lines:&[String],input:&str,blink:bool,scroll:usize,
    ){
        let line_h=16usize; let pad_x=10usize; let pad_y=6usize;
        let visible=((y1.saturating_sub(y0+pad_y*2))/line_h).max(1);
        let total=lines.len();
        let start=total.saturating_sub(visible+scroll);
        let end=total.saturating_sub(scroll).min(total);

        for (row,line) in lines[start..end].iter().enumerate() {
            let fy=y0+pad_y+row*line_h;
            if fy+line_h>y1{break;}
            let col:[u8;3]=if line.starts_with("user@trimangees"){[60,200,80]}
                else if line.starts_with("stderr:")||line.starts_with("error:"){[220,75,75]}
                else if line.starts_with("exit code:"){[160,85,85]}
                else if line.starts_with("warning")||line.starts_with("Warning"){[220,175,50]}
                else if line.starts_with('/')||line.starts_with('~'){[100,160,255]}
                else if line.starts_with("total ")||line.starts_with("drwx")||line.starts_with("-rw"){[140,145,185]}
                else{[188,192,228]};

            if line.starts_with("user@trimangees:~$ ") {
                let after=&line["user@trimangees:~$ ".len()..];
                font.draw(frame,x0+pad_x,fy,"user@trimangees",12.0,[60,200,80]);
                let pw=font.measure("user@trimangees",12.0) as usize;
                font.draw(frame,x0+pad_x+pw,fy,":~$ ",12.0,[110,115,155]);
                let pw2=pw+font.measure(":~$ ",12.0) as usize;
                let ct=font.truncate(after,12.0,(x1.saturating_sub(x0+pad_x+pw2+pad_x)) as f32);
                font.draw(frame,x0+pad_x+pw2,fy,&ct,12.0,[215,220,252]);
            } else {
                let spans=parse_ansi(line,col);
                let max_x=x1.saturating_sub(pad_x); let mut cx=x0+pad_x;
                for span in &spans {
                    if cx>=max_x{break;}
                    let text=font.truncate(&span.text,12.0,(max_x-cx) as f32);
                    if !text.is_empty(){font.draw(frame,cx,fy,&text,12.0,span.col);cx+=font.measure(&text,12.0) as usize;}
                }
            }
        }

        // Input line
        let input_y=y1.saturating_sub(line_h+pad_y);
        if input_y<y1 {
            for x in x0..x1 {
                let sy=input_y.saturating_sub(4);
                if x<SW&&sy<SH{let i=(sy*SW+x)*4;frame[i]=28;frame[i+1]=30;frame[i+2]=50;frame[i+3]=255;}
            }
            font.draw(frame,x0+pad_x,input_y,"user@trimangees",12.0,[60,200,80]);
            let pw=font.measure("user@trimangees",12.0) as usize;
            font.draw(frame,x0+pad_x+pw,input_y,":~$ ",12.0,[110,115,155]);
            let pw2=pw+font.measure(":~$ ",12.0) as usize;
            font.draw(frame,x0+pad_x+pw2,input_y,input,12.0,[215,220,252]);
            if blink {
                let cx=x0+pad_x+pw2+font.measure(input,12.0) as usize;
                for cy in input_y..(input_y+13).min(y1) { for cw in cx..(cx+2).min(x1) {
                    if cw<SW&&cy<SH{let i=(cy*SW+cw)*4;frame[i]=215;frame[i+1]=220;frame[i+2]=252;frame[i+3]=255;}
                }}
            }
        }

        // Scroll indicator
        if scroll>0&&total>visible {
            let bar_x=x1.saturating_sub(4);
            let bar_h=y1.saturating_sub(y0+pad_y*2);
            let thumb_h=((visible as f32/total as f32)*bar_h as f32) as usize;
            let thumb_h=thumb_h.max(16).min(bar_h);
            let max_scroll=total.saturating_sub(visible);
            let thumb_top=y0+pad_y+((scroll as f32/max_scroll as f32)*(bar_h-thumb_h) as f32) as usize;
            for y in (y0+pad_y)..(y0+pad_y+bar_h).min(y1) { for x in bar_x..(bar_x+3).min(SW) {
                if y<SH{let i=(y*SW+x)*4;frame[i]=28;frame[i+1]=30;frame[i+2]=52;frame[i+3]=255;}
            }}
            for y in thumb_top..(thumb_top+thumb_h).min(y1) { for x in bar_x..(bar_x+3).min(SW) {
                if y<SH{let i=(y*SW+x)*4;frame[i]=75;frame[i+1]=80;frame[i+2]=130;frame[i+3]=255;}
            }}
        }
    }

    // ── File Explorer ─────────────────────────────────────────────────
    fn draw_explorer(
        frame:&mut[u8],x0:usize,y0:usize,x1:usize,y1:usize,
        font:&FontRenderer,exp:&crate::window_manager::ExplorerState,
    ){
        let sidebar_w=140usize;
        let addr_h=28usize;
        let row_h=22usize;
        let content_x=x0+sidebar_w+1;

        // ── Sidebar ──────────────────────────────────────────────────
        for y in y0..y1 { for x in x0..(x0+sidebar_w).min(x1) {
            if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=14;frame[i+1]=14;frame[i+2]=22;frame[i+3]=255;}
        }}
        // Sidebar divider
        for y in y0..y1 {
            let sx=(x0+sidebar_w).min(SW-1);
            if y<SH{let i=(y*SW+sx)*4;frame[i]=35;frame[i+1]=38;frame[i+2]=62;frame[i+3]=255;}
        }

        let bookmarks=[
            ("⌂ Home",     std::env::var("HOME").unwrap_or("/home".to_string())),
            ("🖥 Desktop",  format!("{}/Desktop", std::env::var("HOME").unwrap_or_default())),
            ("📄 Documents",format!("{}/Documents",std::env::var("HOME").unwrap_or_default())),
            ("⬇ Downloads", format!("{}/Downloads",std::env::var("HOME").unwrap_or_default())),
            ("/ Root",      "/".to_string()),
            ("/etc",        "/etc".to_string()),
            ("/usr/bin",    "/usr/bin".to_string()),
        ];
        for (i,(lbl,path)) in bookmarks.iter().enumerate() {
            let fy=y0+6+i*22;
            let is_current=path==&exp.current_path;
            if is_current {
                for y in fy..(fy+20).min(y1) { for x in (x0+2)..(x0+sidebar_w-2).min(SW) {
                    if y<SH{let i=(y*SW+x)*4;frame[i]=28;frame[i+1]=32;frame[i+2]=55;frame[i+3]=255;}
                }}
            }
            let col=if is_current{[120u8,165,255]}else{[148,152,190]};
            font.draw(frame,x0+8,fy+5,lbl,11.0,col);
        }

        // ── Address bar ───────────────────────────────────────────────
        // Back / Forward / Up buttons
        let btn_col=[62u8,65,100];
        for y in y0..(y0+addr_h).min(y1) { for x in content_x..(content_x+addr_h*3).min(x1) {
            if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=18;frame[i+1]=18;frame[i+2]=28;frame[i+3]=255;}
        }}
        font.draw(frame,content_x+6, y0+8,"←",12.0,btn_col);
        font.draw(frame,content_x+30,y0+8,"→",12.0,btn_col);
        font.draw(frame,content_x+54,y0+8,"↑",12.0,btn_col);

        // Path bar
        let path_x=content_x+addr_h*3+4;
        for y in y0..(y0+addr_h).min(y1) { for x in path_x..x1.min(SW) {
            if y<SH{let i=(y*SW+x)*4;frame[i]=20;frame[i+1]=20;frame[i+2]=32;frame[i+3]=255;}
        }}
        let path_txt=font.truncate(&exp.current_path,11.5,(x1.saturating_sub(path_x+8)) as f32);
        font.draw(frame,path_x+6,y0+8,&path_txt,11.5,[150,155,200]);

        // Address bar bottom border
        let abord=y0+addr_h;
        for x in content_x..x1 {
            if x<SW&&abord<SH{let i=(abord*SW+x)*4;frame[i]=28;frame[i+1]=30;frame[i+2]=52;frame[i+3]=255;}
        }

        // ── Column headers ─────────────────────────────────────────────
        let hdr_y=abord+1;
        for y in hdr_y..(hdr_y+20).min(y1) { for x in content_x..x1 {
            if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=18;frame[i+1]=18;frame[i+2]=28;frame[i+3]=255;}
        }}
        font.draw(frame,content_x+28,hdr_y+5,"Name",10.5,[90,93,130]);
        font.draw(frame,x1.saturating_sub(120),hdr_y+5,"Size",10.5,[90,93,130]);
        font.draw(frame,x1.saturating_sub(50),hdr_y+5,"Type",10.5,[90,93,130]);
        let hdr_border=hdr_y+20;
        for x in content_x..x1 {
            if x<SW&&hdr_border<SH{let i=(hdr_border*SW+x)*4;frame[i]=28;frame[i+1]=30;frame[i+2]=52;frame[i+3]=255;}
        }

        // ── File rows ─────────────────────────────────────────────────
        let list_y0=hdr_border+1;
        let visible=((y1.saturating_sub(list_y0))/row_h).max(1);
        let start=exp.scroll_offset.min(exp.entries.len().saturating_sub(1));
        let end=(start+visible).min(exp.entries.len());

        for (ri,entry) in exp.entries[start..end].iter().enumerate() {
            let ry=list_y0+ri*row_h;
            if ry+row_h>y1{break;}
            let is_sel=exp.selected==Some(start+ri);

            // Row background
            if is_sel {
                for y in ry..(ry+row_h).min(y1) { for x in content_x..x1 {
                    if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=32;frame[i+1]=72;frame[i+2]=168;frame[i+3]=255;}
                }}
            } else if ri%2==0 {
                for y in ry..(ry+row_h).min(y1) { for x in content_x..x1 {
                    if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=18;frame[i+1]=18;frame[i+2]=26;frame[i+3]=255;}
                }}
            }

            // File icon (coloured square)
            let icon_x=content_x+6; let icon_y=ry+4;
            let (ir,ig,ib):(u8,u8,u8)=Self::file_icon_color(entry);
            for y in icon_y..(icon_y+14).min(y1) { for x in icon_x..(icon_x+16).min(SW) {
                if y<SH{let i=(y*SW+x)*4;frame[i]=ir;frame[i+1]=ig;frame[i+2]=ib;frame[i+3]=255;}
            }}
            // Folder tab
            if entry.is_dir {
                for y in icon_y.saturating_sub(3)..(icon_y).min(y1) { for x in icon_x..(icon_x+8).min(SW) {
                    if y<SH{let i=(y*SW+x)*4;
                    frame[i]=(ir as u16+20).min(255) as u8;
                    frame[i+1]=(ig as u16+20).min(255) as u8;
                    frame[i+2]=ib;frame[i+3]=255;}
                }}
            }

            // File name
            let name_col=if is_sel{[230u8,235,255]}else{[185,190,228]};
            let name_txt=font.truncate(&entry.name,11.5,(x1.saturating_sub(content_x+28+130)) as f32);
            font.draw(frame,content_x+28,ry+6,&name_txt,11.5,name_col);

            // Size
            if !entry.is_dir && entry.size>0 {
                let sz=format_size(entry.size);
                let _sw=font.measure(&sz,10.5) as usize;
                font.draw(frame,x1.saturating_sub(120),ry+6,&sz,10.5,[110,115,155]);
            }

            // Type
            let type_str=if entry.is_dir{"Folder"}else{ext_type(&entry.ext)};
            font.draw(frame,x1.saturating_sub(50),ry+6,type_str,10.5,[90,94,130]);
        }

        // Empty state
        if exp.entries.is_empty() {
            font.draw(frame,(x0+content_x)/2+40,(y0+y1)/2,"Empty folder",13.0,[60,64,95]);
        }

        // Scroll bar for file list
        if exp.entries.len()>visible {
            let sb_x=x1.saturating_sub(4);
            let sb_h=y1.saturating_sub(list_y0);
            let th=((visible as f32/exp.entries.len() as f32)*sb_h as f32).max(20.0) as usize;
            let ms=exp.entries.len().saturating_sub(visible);
            let tp=list_y0+((start as f32/ms as f32)*(sb_h-th) as f32) as usize;
            for y in list_y0..y1 { for x in sb_x..(sb_x+3).min(SW) {
                if y<SH{let i=(y*SW+x)*4;frame[i]=25;frame[i+1]=27;frame[i+2]=45;frame[i+3]=255;}
            }}
            for y in tp..(tp+th).min(y1) { for x in sb_x..(sb_x+3).min(SW) {
                if y<SH{let i=(y*SW+x)*4;frame[i]=70;frame[i+1]=75;frame[i+2]=125;frame[i+3]=255;}
            }}
        }

        // Status bar
        let st_y=y1.saturating_sub(20);
        for y in st_y..y1 { for x in content_x..x1 {
            if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=14;frame[i+1]=14;frame[i+2]=22;frame[i+3]=255;}
        }}
        let st_txt=format!("{} items", exp.entries.len());
        font.draw(frame,content_x+8,st_y+5,&st_txt,10.5,[85,88,125]);
    }

    fn file_icon_color(entry: &FileEntry) -> (u8,u8,u8) {
        if entry.is_dir { return (240,180,40); }
        match entry.ext.as_str() {
            "rs"                          => (220,100,60),
            "py"                          => (60,150,255),
            "js"|"ts"                     => (240,200,50),
            "html"|"htm"                  => (220,80,40),
            "css"                         => (60,120,240),
            "json"|"toml"|"yaml"|"yml"    => (160,200,80),
            "txt"|"md"                    => (160,165,200),
            "png"|"jpg"|"jpeg"|"gif"|"svg"=> (180,80,220),
            "mp4"|"mkv"|"avi"             => (80,180,220),
            "mp3"|"flac"|"wav"            => (60,200,160),
            "pdf"                         => (220,60,60),
            "zip"|"tar"|"gz"|"7z"         => (180,140,60),
            _                             => (100,105,145),
        }
    }

    // ── Settings ──────────────────────────────────────────────────────
    fn draw_settings(
        frame:&mut[u8],x0:usize,y0:usize,x1:usize,y1:usize,
        font:&FontRenderer,s:&SettingsState,
    ){
        let sidebar_w=160usize;

        // Sidebar
        for y in y0..y1 { for x in x0..(x0+sidebar_w).min(x1) {
            if x<SW&&y<SH{let i=(y*SW+x)*4;frame[i]=14;frame[i+1]=14;frame[i+2]=22;frame[i+3]=255;}
        }}
        for y in y0..y1 {
            let sx=(x0+sidebar_w).min(SW-1);
            if y<SH{let i=(y*SW+sx)*4;frame[i]=35;frame[i+1]=38;frame[i+2]=62;frame[i+3]=255;}
        }

        let sections=["Display","Personalise","About"];
        for (i,sec) in sections.iter().enumerate() {
            let sy=y0+12+i*36;
            let is_active=s.active_section==i;
            if is_active {
                for y in sy..(sy+28).min(y1) { for x in (x0+4)..(x0+sidebar_w-4).min(SW) {
                    if y<SH{let i=(y*SW+x)*4;frame[i]=28;frame[i+1]=32;frame[i+2]=58;frame[i+3]=255;}
                }}
                // Active indicator bar
                for y in sy..(sy+28).min(y1) {
                    let bx=x0+2;if bx<SW&&y<SH{let i=(y*SW+bx)*4;frame[i]=72;frame[i+1]=138;frame[i+2]=240;frame[i+3]=255;}
                    let bx2=x0+3;if bx2<SW&&y<SH{let i=(y*SW+bx2)*4;frame[i]=72;frame[i+1]=138;frame[i+2]=240;frame[i+3]=255;}
                }
            }
            let col=if is_active{[200u8,208,255]}else{[110,115,155]};
            font.draw(frame,x0+16,sy+9,sec,12.0,col);
        }

        // Content area
        let cx0=x0+sidebar_w+16; let cy0=y0+16;

        match s.active_section {
            0 => {
                // Display settings
                font.draw(frame,cx0,cy0,"Display Settings",15.0,[215,220,255]);
                font.draw(frame,cx0,cy0+30,"Wallpaper Preset",11.5,[130,135,175]);

                // Preset swatches
                let swatches:[([u8;3],[u8;3]);4]=[
                    ([30,10,80],[10,20,140]),
                    ([80,10,30],[140,20,10]),
                    ([10,60,30],[10,10,80]),
                    ([10,10,40],[40,60,140]),
                ];
                for (i,(c1,c2)) in swatches.iter().enumerate() {
                    let sx=cx0+i*56; let sy=cy0+50;
                    for y in sy..(sy+36).min(y1) { for x in sx..(sx+48).min(x1) {
                        let t=(x-sx) as f32/48.0;
                        if x<SW&&y<SH{let i=(y*SW+x)*4;
                            frame[i]  =(c1[0] as f32*(1.0-t)+c2[0] as f32*t) as u8;
                            frame[i+1]=(c1[1] as f32*(1.0-t)+c2[1] as f32*t) as u8;
                            frame[i+2]=(c1[2] as f32*(1.0-t)+c2[2] as f32*t) as u8;
                            frame[i+3]=255;
                        }
                    }}
                    // Selection border
                    if i==s.wallpaper_preset as usize {
                        for x in sx..(sx+48) { for &dy in &[sy,sy+35] {
                            if x<SW&&dy<SH{let i=(dy*SW+x)*4;frame[i]=72;frame[i+1]=138;frame[i+2]=240;frame[i+3]=255;}
                        }}
                        for y in sy..(sy+36) { for &dx in &[sx,sx+47] {
                            if dx<SW&&y<SH{let i=(y*SW+dx)*4;frame[i]=72;frame[i+1]=138;frame[i+2]=240;frame[i+3]=255;}
                        }}
                    }
                }

                // Accent colour
                font.draw(frame,cx0,cy0+110,"Accent Colour",11.5,[130,135,175]);
                let accent_colors:[[u8;3];6]=[
                    [72,138,240],[60,200,80],[220,100,60],[180,80,220],[240,180,40],[220,60,100]
                ];
                for (i,col) in accent_colors.iter().enumerate() {
                    let ax=cx0+i*36; let ay=cy0+130;
                    let r=12i32; let acx=(ax+16) as i32; let acy=(ay+12) as i32;
                    for dy in -r..=r { for dx in -r..=r {
                        if dx*dx+dy*dy<=r*r {
                            let px=(acx+dx) as usize; let py=(acy+dy) as usize;
                            if px<SW&&py<SH&&px<x1&&py<y1{
                                let i=(py*SW+px)*4;
                                frame[i]=col[0];frame[i+1]=col[1];frame[i+2]=col[2];frame[i+3]=255;
                            }
                        }
                    }}
                    if [s.accent_r,s.accent_g,s.accent_b]==*col {
                        for dy in -(r+2)..=(r+2) { for dx in -(r+2)..=(r+2) {
                            if dx*dx+dy*dy>=(r+1)*(r+1)&&dx*dx+dy*dy<=(r+2)*(r+2) {
                                let px=(acx+dx) as usize; let py=(acy+dy) as usize;
                                if px<SW&&py<SH&&px<x1&&py<y1{
                                    let i=(py*SW+px)*4;
                                    frame[i]=255;frame[i+1]=255;frame[i+2]=255;frame[i+3]=255;
                                }
                            }
                        }}
                    }
                }
            }
            1 => {
                // Personalise
                font.draw(frame,cx0,cy0,"Personalisation",15.0,[215,220,255]);
                font.draw(frame,cx0,cy0+30,"Dark Mode",12.0,[165,170,210]);
                // Toggle
                let tx=cx0+120; let ty=cy0+28;
                for y in ty..(ty+22).min(y1) { for x in tx..(tx+44).min(SW) {
                    let lx=x-tx; let ly=y-ty; let r2=11i32;
                    let _dx=(lx as i32-r2).abs(); let _dy=(ly as i32-r2).abs();
                    if y<SH{let i=(y*SW+x)*4;
                        let c=if s.dark_mode{[32u8,105,220]}else{[60,62,90]};
                        frame[i]=c[0];frame[i+1]=c[1];frame[i+2]=c[2];frame[i+3]=255;
                    }
                }}
                let dot_x=if s.dark_mode{tx+25}else{tx+8};
                let dot_y=cy0+38; let dr=8i32; let dcx=dot_x as i32; let dcy=dot_y as i32;
                for dy2 in -dr..=dr { for dx2 in -dr..=dr {
                    if dx2*dx2+dy2*dy2<=dr*dr {
                        let px=(dcx+dx2) as usize; let py=(dcy+dy2) as usize;
                        if px<SW&&py<SH&&px<x1&&py<y1{let i=(py*SW+px)*4;frame[i]=240;frame[i+1]=242;frame[i+2]=255;frame[i+3]=255;}
                    }
                }}
                let dm_txt=if s.dark_mode{"On"}else{"Off"};
                font.draw(frame,tx+50,ty+6,dm_txt,11.0,[100,105,145]);
            }
            2 => {
                // About
                font.draw(frame,cx0,cy0,"About Trimangees OS",15.0,[215,220,255]);

                let info=[
                    ("OS",          s.os_version.as_str()),
                    ("Hostname",    s.hostname.as_str()),
                    ("Shell",       "Trimangees Shell (Rust)"),
                    ("Renderer",    "pixels + ab_glyph + Inter"),
                    ("Developer",   "Adithya / RoboHolics@GIS"),
                    ("Built with",  "Rust 2024 Edition"),
                ];
                for (i,(label,val)) in info.iter().enumerate() {
                    let iy=cy0+40+i*28;
                    font.draw(frame,cx0,iy,label,11.0,[90,94,130]);
                    font.draw(frame,cx0+110,iy,val,11.5,[185,190,230]);
                }

                // Uptime from /proc/uptime
                if let Ok(ut)=std::fs::read_to_string("/proc/uptime") {
                    if let Some(secs)=ut.split('.').next().and_then(|s|s.parse::<u64>().ok()) {
                        let h=secs/3600; let m=(secs%3600)/60; let s2=secs%60;
                        let uptime=format!("{}h {}m {}s",h,m,s2);
                        font.draw(frame,cx0,cy0+40+6*28,"Uptime",11.0,[90,94,130]);
                        font.draw(frame,cx0+110,cy0+40+6*28,&uptime,11.5,[185,190,230]);
                    }
                }
            }
            _ => {}
        }
    }

    // ── Placeholder for webview apps ──────────────────────────────────
    fn draw_placeholder(frame:&mut[u8],x0:usize,y0:usize,x1:usize,y1:usize,font:&FontRenderer,app:&str){
        let (title,sub,col)=match app {
            "tridocs"   =>("TRiDOCS","Word Processor — opens in system browser on Linux",[70u8,130,220]),
            "trisheets" =>("TRiSHEETS","Spreadsheet — opens in system browser on Linux",[50,180,100]),
            "trislides" =>("TRiSLIDES","Presentations — opens in system browser on Linux",[220,100,60]),
            "tridraw"   =>("TRiDRAW","Drawing Canvas — opens in system browser on Linux",[180,80,220]),
            "browser"   =>("Browser","Opens trimangees.netlify.app in system browser",[60,150,255]),
            _           =>(app,"Placeholder window",[130,135,175]),
        };
        let mx=(x0+x1)/2; let my=(y0+y1)/2;
        // Coloured icon circle
        let r=32i32;
        for dy in -r..=r { for dx in -r..=r {
            if dx*dx+dy*dy<=r*r {
                let px=(mx as i32+dx) as usize; let py=(my as i32-20+dy) as usize;
                if px<SW&&py<SH&&px<x1&&py<y1{let i=(py*SW+px)*4;frame[i]=col[0];frame[i+1]=col[1];frame[i+2]=col[2];frame[i+3]=255;}
            }
        }}
        let tw=font.measure(title,16.0) as usize;
        font.draw(frame,mx.saturating_sub(tw/2),my+20,title,16.0,[215,220,255]);
        let sw=font.measure(sub,10.5) as usize;
        font.draw(frame,mx.saturating_sub(sw/2),my+42,sub,10.5,[100,105,145]);
    }
}

// ── ANSI parser ───────────────────────────────────────────────────────
#[derive(Clone)]
struct Span { text:String, col:[u8;3] }

fn parse_ansi(line:&str, default_col:[u8;3]) -> Vec<Span> {
    let mut spans=Vec::new(); let mut col=default_col; let mut buf=String::new();
    let bytes=line.as_bytes(); let mut i=0;
    while i<bytes.len() {
        if bytes[i]==0x1b && i+1<bytes.len() && bytes[i+1]==b'[' {
            if !buf.is_empty(){spans.push(Span{text:buf.clone(),col});buf.clear();}
            i+=2;
            let mut seq=String::new();
            while i<bytes.len()&&bytes[i]!=b'm' {
                if bytes[i].is_ascii_alphabetic()&&bytes[i]!=b'm'{break;}
                seq.push(bytes[i] as char); i+=1;
            }
            if i<bytes.len(){i+=1;}
            let codes:Vec<u8>=seq.split(';').filter_map(|s|s.parse().ok()).collect();
            let mut ci=0;
            while ci<codes.len() {
                match codes[ci] {
                    0  => col=default_col,
                    1|22 => {}
                    30 => col=[12,12,12],   31 => col=[220,70,70],
                    32 => col=[60,200,80],  33 => col=[220,180,50],
                    34 => col=[80,140,255], 35 => col=[180,80,220],
                    36 => col=[60,190,210], 37 => col=[200,205,230],
                    39 => col=default_col,
                    90 => col=[80,82,110],  91 => col=[255,110,110],
                    92 => col=[100,230,100],93 => col=[255,220,80],
                    94 => col=[110,170,255],95 => col=[210,110,255],
                    96 => col=[90,220,240], 97 => col=[240,242,255],
                    38 if ci+2<codes.len()&&codes[ci+1]==5 => {col=ansi_256(codes[ci+2]);ci+=2;}
                    40..=49|100..=109 => {}
                    _ => {}
                }
                ci+=1;
            }
        } else { buf.push(bytes[i] as char); i+=1; }
    }
    if !buf.is_empty(){spans.push(Span{text:buf,col});}
    if spans.is_empty(){spans.push(Span{text:String::new(),col:default_col});}
    spans
}

fn ansi_256(n:u8)->[u8;3]{
    match n {
        0=>[12,12,12],1=>[197,15,31],2=>[19,161,14],3=>[193,156,0],
        4=>[0,55,218],5=>[136,23,152],6=>[58,150,221],7=>[204,204,204],
        8=>[118,118,118],9=>[231,72,86],10=>[22,198,12],11=>[249,241,165],
        12=>[59,120,255],13=>[180,0,158],14=>[97,214,214],15=>[242,242,242],
        16..=231 => { let idx=n-16; [(idx/36)*51,((idx/6)%6)*51,(idx%6)*51] }
        232..=255 => { let v=8+(n-232)*10; [v,v,v] }
    }
}

fn format_size(bytes:u64)->String {
    if bytes<1024{format!("{}B",bytes)}
    else if bytes<1024*1024{format!("{:.1}KB",bytes as f64/1024.0)}
    else if bytes<1024*1024*1024{format!("{:.1}MB",bytes as f64/1024.0/1024.0)}
    else{format!("{:.1}GB",bytes as f64/1024.0/1024.0/1024.0)}
}

fn ext_type(ext:&str)->&'static str {
    match ext {
        "rs"=>"Rust","py"=>"Python","js"=>"JavaScript","ts"=>"TypeScript",
        "html"|"htm"=>"HTML","css"=>"CSS","json"=>"JSON","toml"=>"TOML",
        "txt"=>"Text","md"=>"Markdown","png"=>"PNG","jpg"|"jpeg"=>"JPEG",
        "gif"=>"GIF","svg"=>"SVG","mp4"=>"Video","mp3"=>"Audio",
        "pdf"=>"PDF","zip"=>"ZIP","tar"=>"TAR","gz"=>"GZip",
        ""=>"",_=>"File",
    }
}

fn lerp(a:u8,b:u8,t:f32)->u8{ (a as f32*(1.0-t)+b as f32*t) as u8 }
/// Akryon OS — Linear Framebuffer Console
/// Double-buffered pixel-level rendering, embedded Terminus 8×16 bitmap font,
/// TrueColor (RGB888) text rendering, and graphical terminal emulation.

use crate::font::{FONT_8X16, FONT_W, FONT_H, FONT_GLYPHS};

extern "C" {
    fn fb_is_active() -> u8;
    fn fb_get_info() -> *const BootInfoC;
}

#[repr(C)]
struct BootInfoC {
    magic:        u32,
    fb_addr:      u32,
    fb_width:     u16,
    fb_height:    u16,
    fb_pitch:     u16,
    fb_bpp:       u8,
    is_graphical: u8,
    reserved:     [u8; 2],
}

// ── Color palette: Catppuccin Mocha TrueColor ───────────────────────────────

#[allow(dead_code)]
pub mod palette {
    pub const BASE:     u32 = 0x000000; // Pure Black
    pub const MANTLE:   u32 = 0x000000; // Pure Black
    pub const SURFACE0: u32 = 0x181825;
    pub const SURFACE1: u32 = 0x313244;
    pub const TEXT:     u32 = 0xCDD6F4;
    pub const SUBTEXT:  u32 = 0xA6ADC8;
    pub const OVERLAY:  u32 = 0x6C7086;
    pub const GREEN:    u32 = 0xA6E3A1;
    pub const TEAL:     u32 = 0x94E2D5;
    pub const SKY:      u32 = 0x89DCEB;
    pub const SAPPHIRE: u32 = 0x74C7EC;
    pub const BLUE:     u32 = 0x89B4FA;
    pub const LAVENDER: u32 = 0xB4BEFE;
    pub const MAUVE:    u32 = 0xCBA6F7;
    pub const PINK:     u32 = 0xF5C2E7;
    pub const RED:      u32 = 0xF38BA8;
    pub const YELLOW:   u32 = 0xF9E2AF;
    pub const PEACH:    u32 = 0xFAB387;
    pub const FLAMINGO: u32 = 0xF2CDCD;
}

// ── VGA-compatible 16-color mapping to TrueColor ────────────────────────────

pub const COLOR_TABLE: [u32; 16] = [
    0x000000,            // 0 Black (Pure Black)
    palette::BLUE,       // 1 Blue
    palette::GREEN,      // 2 Green
    palette::TEAL,       // 3 Cyan
    palette::RED,        // 4 Red
    palette::MAUVE,      // 5 Magenta
    palette::PEACH,      // 6 Brown
    palette::SUBTEXT,    // 7 Light Gray
    palette::OVERLAY,    // 8 Dark Gray
    palette::SAPPHIRE,   // 9 Light Blue
    palette::GREEN,      // 10 Light Green
    palette::SKY,        // 11 Light Cyan
    palette::FLAMINGO,   // 12 Light Red
    palette::LAVENDER,   // 13 Light Magenta
    palette::YELLOW,     // 14 Yellow
    palette::TEXT,       // 15 White
];

// ── Global framebuffer state ─────────────────────────────────────────────────

static mut FB_BASE:   *mut u32 = core::ptr::null_mut();
static mut FB_WIDTH:  usize = 0;
static mut FB_HEIGHT: usize = 0;
static mut FB_PITCH:  usize = 0; // bytes per row
static mut COLS:      usize = 0;
static mut ROWS:      usize = 0;
static mut CUR_X:     usize = 0;
static mut CUR_Y:     usize = 0;
static mut LAST_CUR_X: usize = 0;
static mut LAST_CUR_Y: usize = 0;
static mut CURSOR_ON:  bool = false;
static mut FG_COLOR:  u32 = palette::TEXT;
static mut BG_COLOR:  u32 = 0x000000;
static mut FB_ACTIVE: bool = false;

pub const MOUSE_CURSOR_W: usize = 12;
pub const MOUSE_CURSOR_H: usize = 18;

const MOUSE_CURSOR_SPRITE: [[u8; MOUSE_CURSOR_W]; MOUSE_CURSOR_H] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0],
    [1, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0],
    [1, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0],
];

static mut MOUSE_CURSOR_VISIBLE: bool = false;
static mut MOUSE_PREV_X: usize = 0;
static mut MOUSE_PREV_Y: usize = 0;
static mut MOUSE_SAVED_BG: [u32; MOUSE_CURSOR_W * MOUSE_CURSOR_H] = [0; MOUSE_CURSOR_W * MOUSE_CURSOR_H];

/// One-time initialization — must be called after VMM identity-mapping is live.
pub fn init() {
    unsafe {
        if fb_is_active() == 0 {
            crate::logln!("[FB] fb_is_active=0, skipping LFB init.");
            return;
        }
        let info = &*fb_get_info();

        let phys = info.fb_addr as usize;
        let pitch_bytes = info.fb_pitch as usize;
        let w = info.fb_width as usize;
        let h = info.fb_height as usize;
        let bpp = info.fb_bpp;

        crate::logln!("[FB] phys=0x{:08X} pitch={} w={} h={} bpp={}", phys, pitch_bytes, w, h, bpp);

        if bpp != 32 {
            crate::logln!("[FB] bpp!=32, aborting.");
            return;
        }

        FB_BASE   = phys as *mut u32;
        FB_WIDTH  = w;
        FB_HEIGHT = h;
        let p_words = pitch_bytes / 4;
        let c_val = w / FONT_W;
        let r_val = h / FONT_H;
        FB_PITCH  = p_words; // pitch in u32 words
        COLS      = c_val;
        ROWS      = r_val;

        crate::logln!("[FB] FB_BASE=0x{:08X} PITCH={} COLS={} ROWS={}", phys, p_words, c_val, r_val);

        // Write a single test pixel to verify the LFB mapping works
        crate::logln!("[FB] Writing test pixel...");
        *FB_BASE = 0x001E1E2E_u32;
        crate::logln!("[FB] Test pixel OK. Clearing screen...");

        FB_ACTIVE = true;
        clear_screen_color(BG_COLOR);
    }
    crate::logln!("[FB] Linear framebuffer console ready ({}x{})", unsafe { FB_WIDTH }, unsafe { FB_HEIGHT });
}

#[inline]
pub fn is_active() -> bool {
    unsafe { FB_ACTIVE }
}

// ── Pixel primitives ─────────────────────────────────────────────────────────

#[inline]
pub fn put_pixel(x: usize, y: usize, color: u32) {
    unsafe {
        if x < FB_WIDTH && y < FB_HEIGHT {
            *FB_BASE.add(y * FB_PITCH + x) = color;
        }
    }
}

pub fn fill_rect(x: usize, y: usize, w: usize, h: usize, color: u32) {
    unsafe {
        let x_end = (x + w).min(FB_WIDTH);
        let y_end = (y + h).min(FB_HEIGHT);
        for row in y..y_end {
            let base = FB_BASE.add(row * FB_PITCH + x);
            for col in 0..(x_end - x) {
                *base.add(col) = color;
            }
        }
    }
}

pub fn clear_screen_color(color: u32) {
    unsafe {
        let total = FB_HEIGHT * FB_PITCH;
        let p = FB_BASE;
        for i in 0..total {
            *p.add(i) = color;
        }
        CUR_X = 0;
        CUR_Y = 0;
    }
}

// ── Font rasterizer ──────────────────────────────────────────────────────────

fn draw_glyph(c: u8, px: usize, py: usize, fg: u32, bg: u32) {
    let glyph_idx = if (c as usize) < FONT_GLYPHS { c as usize } else { 0x3F };
    let glyph = &FONT_8X16[glyph_idx * FONT_H..(glyph_idx + 1) * FONT_H];

    unsafe {
        for row in 0..FONT_H {
            let bits = glyph[row];
            let row_base = FB_BASE.add((py + row) * FB_PITCH + px);
            for col in 0..FONT_W {
                let on = (bits >> (7 - col)) & 1 != 0;
                *row_base.add(col) = if on { fg } else { bg };
            }
        }
    }
}

// ── Terminal scrolling ───────────────────────────────────────────────────────

pub fn hide_cursor() {
    unsafe {
        if !FB_ACTIVE || !CURSOR_ON { return; }
        let px = LAST_CUR_X * FONT_W;
        let py = LAST_CUR_Y * FONT_H + 14;
        for col in 0..FONT_W {
            put_pixel(px + col, py, BG_COLOR);
            put_pixel(px + col, py + 1, BG_COLOR);
        }
        CURSOR_ON = false;
    }
}

pub fn show_cursor() {
    unsafe {
        if !FB_ACTIVE { return; }
        hide_cursor();
        let px = CUR_X * FONT_W;
        let py = CUR_Y * FONT_H + 14;
        let cursor_color = if FG_COLOR == BG_COLOR { palette::TEXT } else { FG_COLOR };
        for col in 0..FONT_W {
            put_pixel(px + col, py, cursor_color);
            put_pixel(px + col, py + 1, cursor_color);
        }
        LAST_CUR_X = CUR_X;
        LAST_CUR_Y = CUR_Y;
        CURSOR_ON = true;
    }
}

fn scroll_up_one() {
    unsafe {
        hide_cursor();
        hide_mouse_cursor();
        let row_pixels = FONT_H * FB_PITCH;
        let text_rows = ROWS.saturating_sub(1);
        let total_words = text_rows * row_pixels;
        let src = FB_BASE.add(row_pixels);
        let dst = FB_BASE;

        for i in 0..total_words {
            *dst.add(i) = *src.add(i);
        }

        let last_row_start = FB_BASE.add(total_words);
        for i in 0..row_pixels {
            *last_row_start.add(i) = BG_COLOR;
        }

        if CUR_Y > 0 {
            CUR_Y -= 1;
        }
    }
}

// ── Public text output ───────────────────────────────────────────────────────

pub fn set_color(fg: u32, bg: u32) {
    unsafe {
        FG_COLOR = fg;
        BG_COLOR = bg;
    }
}

pub fn get_fg() -> u32 { unsafe { FG_COLOR } }
pub fn get_bg() -> u32 { unsafe { BG_COLOR } }

pub fn putchar(c: u8) {
    unsafe {
        hide_cursor();
        match c {
            b'\n' => {
                CUR_X = 0;
                CUR_Y += 1;
                if CUR_Y >= ROWS {
                    scroll_up_one();
                }
            }
            b'\r' => {
                CUR_X = 0;
            }
            b'\t' => {
                CUR_X = (CUR_X + 4) & !3;
                if CUR_X >= COLS {
                    CUR_X = 0;
                    CUR_Y += 1;
                    if CUR_Y >= ROWS {
                        scroll_up_one();
                    }
                }
            }
            b'\x08' => {
                // backspace
                if CUR_X > 0 {
                    CUR_X -= 1;
                } else if CUR_Y > 0 {
                    CUR_Y -= 1;
                    CUR_X = COLS - 1;
                }
                draw_glyph(b' ', CUR_X * FONT_W, CUR_Y * FONT_H, FG_COLOR, BG_COLOR);
            }
            _ => {
                if CUR_X >= COLS {
                    CUR_X = 0;
                    CUR_Y += 1;
                    if CUR_Y >= ROWS {
                        scroll_up_one();
                    }
                }
                draw_glyph(c, CUR_X * FONT_W, CUR_Y * FONT_H, FG_COLOR, BG_COLOR);
                CUR_X += 1;
            }
        }
        show_cursor();
    }
}

pub fn putchar_at(c: u8, fg: u32, bg: u32, col: usize, row: usize) {
    unsafe {
        if !FB_ACTIVE || col >= COLS || row >= ROWS { return; }
        draw_glyph(c, col * FONT_W, row * FONT_H, fg, bg);
    }
}

pub fn puts(s: &str) {
    for b in s.bytes() {
        putchar(b);
    }
}

pub fn clear_screen() {
    unsafe {
        hide_cursor();
        hide_mouse_cursor();
        clear_screen_color(BG_COLOR);
        show_cursor();
    }
}

pub fn get_cursor() -> (usize, usize) {
    unsafe { (CUR_X, CUR_Y) }
}

pub fn set_cursor(x: usize, y: usize) {
    unsafe {
        hide_cursor();
        CUR_X = x.min(COLS.saturating_sub(1));
        CUR_Y = y.min(ROWS.saturating_sub(1));
        show_cursor();
    }
}

pub fn cols() -> usize { unsafe { COLS } }
pub fn rows() -> usize { unsafe { ROWS } }
pub fn width() -> usize { unsafe { FB_WIDTH } }
pub fn height() -> usize { unsafe { FB_HEIGHT } }

pub fn get_pixel(x: usize, y: usize) -> u32 {
    unsafe {
        if FB_ACTIVE && x < FB_WIDTH && y < FB_HEIGHT {
            *FB_BASE.add(y * FB_PITCH + x)
        } else {
            0
        }
    }
}

pub fn hide_mouse_cursor() {
    unsafe {
        if !FB_ACTIVE || !MOUSE_CURSOR_VISIBLE {
            return;
        }
        for row in 0..MOUSE_CURSOR_H {
            let py = MOUSE_PREV_Y + row;
            if py >= FB_HEIGHT {
                break;
            }
            for col in 0..MOUSE_CURSOR_W {
                let px = MOUSE_PREV_X + col;
                if px < FB_WIDTH {
                    *FB_BASE.add(py * FB_PITCH + px) = MOUSE_SAVED_BG[row * MOUSE_CURSOR_W + col];
                }
            }
        }
        MOUSE_CURSOR_VISIBLE = false;
    }
}

pub fn render_mouse_cursor(new_x: usize, new_y: usize) {
    unsafe {
        if !FB_ACTIVE {
            return;
        }
        if MOUSE_CURSOR_VISIBLE && (MOUSE_PREV_X != new_x || MOUSE_PREV_Y != new_y) {
            hide_mouse_cursor();
        }

        if !MOUSE_CURSOR_VISIBLE {
            for row in 0..MOUSE_CURSOR_H {
                let py = new_y + row;
                if py >= FB_HEIGHT {
                    break;
                }
                for col in 0..MOUSE_CURSOR_W {
                    let px = new_x + col;
                    if px < FB_WIDTH {
                        MOUSE_SAVED_BG[row * MOUSE_CURSOR_W + col] = *FB_BASE.add(py * FB_PITCH + px);
                    } else {
                        MOUSE_SAVED_BG[row * MOUSE_CURSOR_W + col] = 0;
                    }
                }
            }

            for row in 0..MOUSE_CURSOR_H {
                let py = new_y + row;
                if py >= FB_HEIGHT {
                    break;
                }
                for col in 0..MOUSE_CURSOR_W {
                    let px = new_x + col;
                    if px < FB_WIDTH {
                        match MOUSE_CURSOR_SPRITE[row][col] {
                            1 => *FB_BASE.add(py * FB_PITCH + px) = 0x000000,
                            2 => *FB_BASE.add(py * FB_PITCH + px) = palette::TEXT,
                            _ => {}
                        }
                    }
                }
            }

            MOUSE_PREV_X = new_x;
            MOUSE_PREV_Y = new_y;
            MOUSE_CURSOR_VISIBLE = true;
        }
    }
}

// ── Gradient fills (for lalaufetch) ─────────────────────────────────────────

fn lerp_color(a: u32, b: u32, t: usize, total: usize) -> u32 {
    if total == 0 { return a; }
    let ar = ((a >> 16) & 0xFF) as usize;
    let ag = ((a >> 8) & 0xFF) as usize;
    let ab = (a & 0xFF) as usize;
    let br = ((b >> 16) & 0xFF) as usize;
    let bg = ((b >> 8) & 0xFF) as usize;
    let bb = (b & 0xFF) as usize;
    let r = ar + (br.wrapping_sub(ar).wrapping_mul(t)) / total;
    let g = ag + (bg.wrapping_sub(ag).wrapping_mul(t)) / total;
    let bl = ab + (bb.wrapping_sub(ab).wrapping_mul(t)) / total;
    ((r as u32) << 16) | ((g as u32) << 8) | (bl as u32)
}

pub fn gradient_h_rect(x: usize, y: usize, w: usize, h: usize, c_left: u32, c_right: u32) {
    for row in y..y.saturating_add(h).min(unsafe { FB_HEIGHT }) {
        for col in 0..w {
            let c = lerp_color(c_left, c_right, col, w.saturating_sub(1));
            put_pixel(x + col, row, c);
        }
    }
}

/// Draw a Catppuccin-themed horizontal banner line behind a text row.
pub fn draw_banner_bg(row: usize, c_left: u32, c_right: u32) {
    unsafe {
        gradient_h_rect(0, row * FONT_H, FB_WIDTH, FONT_H, c_left, c_right);
    }
}

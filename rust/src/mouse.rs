extern "C" {
    fn mouse_set_bounds(max_x: i32, max_y: i32);
    fn mouse_get_state(x: *mut i32, y: *mut i32, buttons: *mut u8);
    fn mouse_get_x() -> i32;
    fn mouse_get_y() -> i32;
    fn mouse_get_buttons() -> u8;
    fn mouse_get_event_count() -> u32;
}

pub const BUTTON_LEFT: u8 = 1 << 0;
pub const BUTTON_RIGHT: u8 = 1 << 1;
pub const BUTTON_MIDDLE: u8 = 1 << 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub buttons: u8,
}

pub fn set_bounds(width: usize, height: usize) {
    let max_x = if width > 0 { (width - 1) as i32 } else { 0 };
    let max_y = if height > 0 { (height - 1) as i32 } else { 0 };
    unsafe {
        mouse_set_bounds(max_x, max_y);
    }
}

pub fn get_state() -> MouseState {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut buttons: u8 = 0;

    unsafe {
        mouse_get_state(&mut x, &mut y, &mut buttons);
    }

    MouseState {
        x,
        y,
        left: (buttons & BUTTON_LEFT) != 0,
        right: (buttons & BUTTON_RIGHT) != 0,
        middle: (buttons & BUTTON_MIDDLE) != 0,
        buttons,
    }
}

pub fn get_position() -> (i32, i32) {
    unsafe { (mouse_get_x(), mouse_get_y()) }
}

pub fn get_event_count() -> u32 {
    unsafe { mouse_get_event_count() }
}

pub fn is_left_down() -> bool {
    unsafe { (mouse_get_buttons() & BUTTON_LEFT) != 0 }
}

pub fn is_right_down() -> bool {
    unsafe { (mouse_get_buttons() & BUTTON_RIGHT) != 0 }
}

pub fn is_middle_down() -> bool {
    unsafe { (mouse_get_buttons() & BUTTON_MIDDLE) != 0 }
}

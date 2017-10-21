////////////////////////////////////////////////////////////////////////////////
// Utilities
////////////////////////////////////////////////////////////////////////////////

#[inline]
pub fn hex_char(nibble: u8) -> u8 {
    if nibble < 10 {
        48 + nibble
    } else {
        97 + nibble - 10
    }
}

use colors::Style;
use colors;
use termbox_simple::*;

pub fn draw_box(tb: &mut Termbox, pos_x: i32, pos_y: i32, width: i32, height: i32) {
    let fg = colors::DEFAULT.fg;
    let bg = colors::DEFAULT.bg;

    for x in 1..width - 1 {
        tb.change_cell(pos_x + x, pos_y, '─', fg, bg);
        tb.change_cell(pos_x + x, pos_y + height - 1, '─', fg, bg);
    }

    for y in 1..height - 1 {
        tb.change_cell(pos_x, pos_y + y, '│', fg, bg);
        tb.change_cell(pos_x + width - 1, pos_y + y, '│', fg, bg);
    }

    tb.change_cell(pos_x, pos_y, '┌', fg, bg);
    tb.change_cell(pos_x + width - 1, pos_y, '┐', fg, bg);
    tb.change_cell(pos_x, pos_y + height - 1, '└', fg, bg);
    tb.change_cell(pos_x + width - 1, pos_y + height - 1, '┘', fg, bg);

    for x in 1..width - 1 {
        for y in 1..height - 1 {
            tb.change_cell(pos_x + x, pos_y + y, ' ', fg, bg);
        }
    }
}


pub fn print(tb: &mut Termbox, mut pos_x: i32, pos_y: i32, style: Style, str: &str) {
    for char in str.chars() {
        tb.change_cell(pos_x, pos_y, char, style.fg, style.bg);
        pos_x += 1;
    }
}

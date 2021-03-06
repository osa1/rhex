use std::cmp;

use colors;

use termbox_simple::*;

pub struct AsciiView<'view> {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,

    data: &'view [u8],

    cursor_x: i32,
    cursor_y: i32,
    scroll: i32,

    has_focus: bool,
}

impl<'view> AsciiView<'view> {
    pub fn new(
        width: i32,
        height: i32,
        pos_x: i32,
        pos_y: i32,
        data: &'view [u8],
    ) -> AsciiView<'view> {
        AsciiView {
            width: width,
            height: height,
            pos_x: pos_x,
            pos_y: pos_y,
            data: data,
            cursor_x: 0,
            cursor_y: 0,
            scroll: 0,
            has_focus: false,
        }
    }

    pub fn set_scroll(&mut self, scroll: i32) {
        self.scroll = scroll;
    }

    pub fn draw(&self, tb: &mut Termbox, hl: &[usize], hl_len: usize) {
        let rows = self.height;
        let cols = self.width;

        let mut hl_idx = 0;

        'outer: for row in self.scroll..self.scroll + rows {
            for col in 0..cols {
                let byte_idx = (row * cols + col) as usize;
                if let Some(&byte) = self.data.get(byte_idx) {
                    let ch = if byte >= 32 && byte <= 126 {
                        byte
                    } else {
                        b'.'
                    };

                    while hl_idx < hl.len() && hl[hl_idx] + hl_len < byte_idx {
                        hl_idx += 1;
                    }

                    let style = if self.cursor_x == col && self.cursor_y == row {
                        if self.has_focus {
                            colors::CURSOR_FOCUS
                        } else {
                            colors::CURSOR_NO_FOCUS
                        }
                    } else if let Some(&hl_offset) = hl.get(hl_idx) {
                        if byte_idx >= hl_offset && byte_idx < hl_offset + hl_len {
                            colors::HIGHLIGHT
                        } else {
                            colors::DEFAULT
                        }
                    } else {
                        colors::DEFAULT
                    };

                    tb.change_cell(
                        self.pos_x + col,
                        self.pos_y + row - self.scroll,
                        ch as char,
                        style.fg,
                        style.bg,
                    );
                } else {
                    break 'outer;
                }
            }
        }
    }

    pub fn move_cursor_offset(&mut self, byte_idx: i32) {
        let cursor_y = byte_idx / self.width;
        let cursor_x = byte_idx % self.width;

        if cursor_y > self.scroll + self.height - 3 {
            self.scroll = cursor_y - (self.height - 3);
        } else if cursor_y < self.scroll + 2 {
            self.scroll = cmp::max(cursor_y - 2, 0);
        }

        self.cursor_y = cursor_y;
        self.cursor_x = cursor_x;
    }
}

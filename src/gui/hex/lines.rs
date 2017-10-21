use std::borrow::Borrow;
use std::cmp;

use colors::Color;
use utils;

use ncurses as nc;

pub struct Lines {
    bytes_per_line: i32,
    length: i32,

    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,

    /// Byte offset (aka. address)
    cursor: i32,

    scroll: i32,
}

impl Lines {
    pub fn new(
        bytes_per_line: i32,
        length: i32,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
    ) -> Lines {
        Lines {
            bytes_per_line: bytes_per_line,
            length: length,
            pos_x: pos_x,
            pos_y: pos_y,
            width: width,
            height: height,
            cursor: 0,
            scroll: 0,
        }
    }

    pub fn set_scroll(&mut self, scroll: i32) {
        self.scroll = scroll;
    }

    pub fn draw(&self) {
        let mut addr_str = String::with_capacity(self.width as usize);

        let start_addr = self.scroll * self.bytes_per_line;

        for line in 0..self.height {
            let addr = start_addr + self.bytes_per_line * line;
            if addr >= self.length {
                break;
            }

            self.mk_hex_string(addr, &mut addr_str);

            let highlight = self.cursor >= addr && self.cursor < addr + self.bytes_per_line;

            if highlight {
                nc::attron(nc::A_BOLD() | Color::CursorNoFocus.attr());
            }

            nc::mvaddstr(self.pos_y + line, self.pos_x, addr_str.borrow());

            if highlight {
                nc::attroff(nc::A_BOLD() | Color::CursorNoFocus.attr());
            }
        }
    }

    pub fn move_cursor_offset(&mut self, byte_offset: i32) {
        self.cursor = byte_offset;

        let mut line = byte_offset / self.bytes_per_line;
        if byte_offset % self.bytes_per_line != 0 {
            line += 1;
        }

        let min_scroll = cmp::max(0, line - self.height + 3);
        let max_scroll = cmp::max(0, line - 3);

        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        } else if self.scroll < min_scroll {
            self.scroll = min_scroll;
        }
    }

    fn mk_hex_string(&self, addr: i32, ret: &mut String) {
        ret.clear();

        // for debugging purposes:
        // ret.push_str(format!("{}", addr).borrow());

        ret.push('0');
        ret.push('x');

        for i in 0..self.width - 2 + 1 {
            let nibble = ((addr >> (4 * (self.width - 2 - i))) & 0b0000_1111) as u8;
            ret.push(utils::hex_char(nibble) as char);
        }
    }
}

use std::cmp;

use colors;
use utils::*;

use termbox_simple::*;

pub struct Lines {
    bytes_per_line: i32,
    length: i32,

    width: i32,
    height: i32,

    /// Byte offset (aka. address)
    cursor: i32,

    scroll: i32,
}

impl Lines {
    pub fn new(bytes_per_line: i32, length: i32, width: i32, height: i32) -> Lines {
        Lines {
            bytes_per_line: bytes_per_line,
            length: length,
            width: width,
            height: height,
            cursor: 0,
            scroll: 0,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn set_scroll(&mut self, scroll: i32) {
        self.scroll = scroll;
    }

    pub fn draw(&self, tb: &mut Termbox) {
        let mut addr_str = String::with_capacity(self.width as usize);

        let start_addr = self.scroll * self.bytes_per_line;

        for line in 0..self.height {
            let addr = start_addr + self.bytes_per_line * line;
            if addr >= self.length {
                break;
            }

            self.mk_hex_string(addr, &mut addr_str);

            let highlight = self.cursor >= addr && self.cursor < addr + self.bytes_per_line;
            let style = if highlight {
                colors::CURSOR_NO_FOCUS
            } else {
                colors::DEFAULT
            };

            print(tb, 0, line, style, &addr_str);
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
            ret.push(hex_char(nibble) as char);
        }
    }
}

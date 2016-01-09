use std::cmp;

// Is there a better way to make writeln! working with std::io::stderr()?
use std::io::prelude::*;

use colors::Color;
use utils::*;

use ncurses as nc;

pub enum SearchRet {
    /// Highlight these bytes.
    Highlight {
        /// Byte in focus.
        focus: usize,

        /// All matching byte offsets.
        all_bytes: Vec<usize>,

        /// Length of searched bytes.
        len: usize,
    },

    /// User cancelled.
    Abort,

    /// Carry on.
    Continue,
}

enum SearchMode { Ascii, Hex }

pub struct SearchOverlay<'overlay> {
    win: nc::WINDOW,
    width: i32,
    height: i32,

    // TODO: rename this to maybe something like 'focus'
    mode: SearchMode,
    buffer: Vec<u8>,

    /// Byte offset in buffer.
    byte_cursor: usize,

    contents: &'overlay Vec<u8>
}

impl<'overlay> Drop for SearchOverlay<'overlay> {
    fn drop(&mut self) {
        nc::delwin(self.win);
    }
}

impl<'overlay> SearchOverlay<'overlay> {
    pub fn new(width: i32, height: i32, pos_x: i32, pos_y: i32, contents: &'overlay Vec<u8>)
               -> SearchOverlay<'overlay> {
        let width_ = cmp::min(width, 50);
        let height_ = cmp::min(height, 10);

        let pos_x = pos_x + (width - width_) / 2;
        let pos_y = pos_y + (height - height_) / 2;

        SearchOverlay {
            win: nc::newwin(height_, width_, pos_y, pos_x),
            width: width_,
            height: height_,

            mode: SearchMode::Ascii,
            buffer: Vec::new(),
            byte_cursor: 0,

            contents: contents,
        }
    }

    pub fn draw(&self) {
        nc::wclear(self.win);

        nc::box_(self.win, 0, 0);

        nc::mvwaddch(self.win, 0, self.width / 2, nc::ACS_TTEE());
        nc::mvwvline(self.win, 1, self.width / 2, nc::ACS_VLINE(), self.height - 2);
        nc::mvwaddch(self.win, self.height - 1, self.width / 2, nc::ACS_BTEE());

        self.draw_hex();
        self.draw_ascii();

        nc::wrefresh(self.win);
    }

    fn draw_ascii(&self) {
        // Not the most efficient way to draw, but be fine at this scale
        // (e.g. for a couple of characters at most)
        let width = ((self.width - 1) / 2) as usize;
        for (byte_offset, byte) in self.buffer.iter().enumerate() {
            let pos_x = (byte_offset % width) + 1;
            let pos_y = (byte_offset / width) + 1;

            nc::mvwaddch(self.win, pos_y as i32, pos_x as i32, *byte as u64);
        }

        // Draw cursor
        let cursor_x = (self.byte_cursor % width) + 1;
        let cursor_y = self.byte_cursor / width;

        let byte =
            if self.byte_cursor >= self.buffer.len() {
                b' '
            } else {
                self.buffer[self.byte_cursor]
            };

        nc::wattron(self.win, Color::CursorFocus.attr());
        nc::mvwaddch(self.win, cursor_y as i32 + 1, cursor_x as i32, byte as u64);
        nc::wattroff(self.win, Color::CursorFocus.attr());
    }

    fn draw_hex(&self) {
        // Ideally we could reuse some of the code from HexGrid, but the code
        // here should be very simple as we don't have to deal with scrolling,
        // jumping around etc.
        let start_column = self.width / 2;
        let width        = self.width / 2 - 1;

        // We skip first row and column as it's occupied by the window border
        let mut col = 1;
        let mut row = 1;

        for (byte_offset, byte) in self.buffer.iter().enumerate() {
            if col + 1 >= width {
                col  = 1;
                row += 1;
            }

            let nibble1 = hex_char(*byte >> 4);
            let nibble2 = hex_char(*byte & 0b00001111);

            nc::mvwaddch(self.win, row, start_column + col,     nibble1 as u64);
            nc::mvwaddch(self.win, row, start_column + col + 1, nibble2 as u64);

            col += 3;
        }

        // Draw cursor
        let mut bytes_per_line = width / 3;

        let cursor_x_byte = self.byte_cursor as i32 % bytes_per_line;
        let cursor_x      = cursor_x_byte * 3 + 1;
        let cursor_y      = self.byte_cursor as i32 / bytes_per_line;

        let byte =
            if self.byte_cursor >= self.buffer.len() {
                b' '
            } else {
                self.buffer[self.byte_cursor]
            };

        nc::wattron(self.win, Color::CursorNoFocus.attr());
        nc::mvwaddch(self.win, cursor_y + 1, start_column + cursor_x, byte as u64);
        nc::wattroff(self.win, Color::CursorNoFocus.attr());
    }

    pub fn keypressed(&mut self, ch : i32) -> SearchRet {
        // TODO: We should be able to move cursor and insert at the cursor
        // position.

        if ch == 27 {
            // FIXME: This part is copied from goto

            // Is it escape or ALT + something?
            nc::nodelay( self.win, true );
            let next_ch = nc::wgetch( self.win );
            nc::nodelay( self.win, false );

            if next_ch == -1 {
                // It's escape, abort
                SearchRet::Abort
            } else {
                // It's ALT + something, but we don't handle that
                SearchRet::Continue
            }
        }

        else if ch == nc::KEY_BACKSPACE || ch == 127 {
            match self.buffer.pop() {
                None => {},
                Some(_) => { self.byte_cursor -= 1; }
            }
            SearchRet::Continue
        }

        else if ch == 10 || ch == b'\n' as i32 {
            if self.buffer.len() != 0 {
                // do the search
                let offsets = self.find_offsets();
                SearchRet::Highlight {
                    focus: self.byte_cursor,
                    all_bytes: offsets,
                    len: self.buffer.len(),
                }
            } else {
                SearchRet::Continue
            }
        }

        else {
            match self.mode {
                SearchMode::Ascii => {
                    match ch {
                        0 ... 0xFF => {
                            self.buffer.push(ch as u8);
                            self.byte_cursor += 1;
                        },
                        _ => { /* ignore */ },
                    }
                },
                SearchMode::Hex => {

                },
            }

            SearchRet::Continue
        }
    }

    pub fn get_char(&self) -> i32 {
        nc::wgetch(self.win)
    }

    fn find_offsets(&self) -> Vec<usize> {
        let mut ret = Vec::new();

        let first_byte = self.buffer[0];

        // It seems like Vec API doesn't help us here. As a first
        // implementation, I do a O(n * k) search here.
        let mut byte_offset = 0;
        while byte_offset < self.contents.len() {
            let byte = unsafe { *self.contents.get_unchecked(byte_offset) };
            if byte == first_byte {
                if try_match(&self.contents[ byte_offset + 1 ..  ], &self.buffer[ 1 .. ]) {
                    ret.push(byte_offset);
                    byte_offset += self.buffer.len();
                    continue;
                }
            }

            byte_offset += 1;
        }

        // writeln!(&mut ::std::io::stderr(), "find_offsets: {:?}", ret);
        ret
    }
}

fn try_match(s1 : &[u8], s2 : &[u8]) -> bool {
    if s2.len() > s1.len() {
        false
    } else {
        for (byte1, byte2) in s1.iter().zip(s2.iter()) {
            if byte1 != byte2 {
                return false;
            }
        }

        true
    }
}

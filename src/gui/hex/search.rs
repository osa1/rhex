use std::cmp;

use colors::Color;
use utils::*;

use ncurses as nc;
use term_input::Key;

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

enum SearchMode {
    Ascii,
    Hex,
}

enum NibbleCursor {
    /// More significant part
    MS,
    /// Less significant part
    LS,
}

pub struct SearchOverlay<'overlay> {
    win: nc::WINDOW,
    width: i32,
    height: i32,

    // TODO: rename this to maybe something like 'focus'
    mode: SearchMode,
    buffer: Vec<u8>,

    /// Byte offset in buffer.
    byte_cursor: usize,
    nibble_cursor: NibbleCursor,

    contents: &'overlay [u8],
}

impl<'overlay> Drop for SearchOverlay<'overlay> {
    fn drop(&mut self) {
        nc::delwin(self.win);
    }
}

impl<'overlay> SearchOverlay<'overlay> {
    pub fn new(
        width: i32,
        height: i32,
        pos_x: i32,
        pos_y: i32,
        contents: &'overlay [u8],
    ) -> SearchOverlay<'overlay> {
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
            nibble_cursor: NibbleCursor::MS,

            contents: contents,
        }
    }

    pub fn draw(&self) {
        nc::wclear(self.win);

        nc::box_(self.win, 0, 0);

        nc::mvwaddch(self.win, 0, self.width / 2, nc::ACS_TTEE());
        nc::mvwvline(
            self.win,
            1,
            self.width / 2,
            nc::ACS_VLINE(),
            self.height - 2,
        );
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

        let byte = if self.byte_cursor >= self.buffer.len() {
            b' '
        } else {
            self.buffer[self.byte_cursor]
        };

        match self.mode {
            SearchMode::Ascii => {
                nc::wattron(self.win, Color::CursorFocus.attr());
            }
            SearchMode::Hex => {
                nc::wattron(self.win, Color::CursorNoFocus.attr());
            }
        }

        nc::mvwaddch(self.win, cursor_y as i32 + 1, cursor_x as i32, byte as u64);

        match self.mode {
            SearchMode::Ascii => {
                nc::wattroff(self.win, Color::CursorFocus.attr());
            }
            SearchMode::Hex => {
                nc::wattroff(self.win, Color::CursorNoFocus.attr());
            }
        }
    }

    fn draw_hex(&self) {
        // Ideally we could reuse some of the code from HexGrid, but the code
        // here should be very simple as we don't have to deal with scrolling,
        // jumping around etc.
        let start_column = self.width / 2;
        let width = self.width / 2 - 1;

        // We skip first row and column as it's occupied by the window border
        let mut col = 1;
        let mut row = 1;

        for byte in &self.buffer {
            if col + 1 >= width {
                col = 1;
                row += 1;
            }

            let nibble1 = hex_char(*byte >> 4);
            let nibble2 = hex_char(*byte & 0b0000_1111);

            nc::mvwaddch(self.win, row, start_column + col, nibble1 as u64);
            nc::mvwaddch(self.win, row, start_column + col + 1, nibble2 as u64);

            col += 3;
        }

        // Draw cursor
        let bytes_per_line = width / 3;

        let cursor_x_byte = self.byte_cursor as i32 % bytes_per_line;
        let cursor_x = cursor_x_byte * 3 + 1;
        let cursor_x = match self.nibble_cursor {
            NibbleCursor::MS =>
                cursor_x,
            NibbleCursor::LS =>
                cursor_x + 1,
        };
        let cursor_y = self.byte_cursor as i32 / bytes_per_line;

        let byte = if self.byte_cursor >= self.buffer.len() {
            b' '
        } else {
            match self.nibble_cursor {
                NibbleCursor::MS =>
                    hex_char(self.buffer[self.byte_cursor] >> 4),
                NibbleCursor::LS =>
                    hex_char(self.buffer[self.byte_cursor] & 0b0000_1111),
            }
        };

        match self.mode {
            SearchMode::Hex => {
                nc::wattron(self.win, Color::CursorFocus.attr());
            }
            SearchMode::Ascii => {
                nc::wattron(self.win, Color::CursorNoFocus.attr());
            }
        }

        nc::mvwaddch(self.win, cursor_y + 1, start_column + cursor_x, byte as u64);

        match self.mode {
            SearchMode::Hex => {
                nc::wattroff(self.win, Color::CursorFocus.attr());
            }
            SearchMode::Ascii => {
                nc::wattroff(self.win, Color::CursorNoFocus.attr());
            }
        }
    }

    pub fn keypressed(&mut self, key: Key) -> SearchRet {
        // TODO: We should be able to move cursor and insert at the cursor
        // position.

        match key {
            Key::Esc => {
                return SearchRet::Abort;
            }
            Key::Enter => {
                if !self.buffer.is_empty() {
                    // do the search
                    let offsets = self.find_offsets();
                    return SearchRet::Highlight {
                        focus: self.byte_cursor,
                        all_bytes: offsets,
                        len: self.buffer.len(),
                    };
                }
            }
            Key::Tab => {
                let new_sm = match self.mode {
                    SearchMode::Ascii =>
                        SearchMode::Hex,
                    SearchMode::Hex =>
                        SearchMode::Ascii,
                };
                self.mode = new_sm;
            }
            Key::Backspace =>
                match self.mode {
                    SearchMode::Ascii =>
                        match self.buffer.pop() {
                            None =>
                                {}
                            Some(_) =>
                                if self.byte_cursor != 0 {
                                    self.byte_cursor -= 1;
                                },
                        },
                    SearchMode::Hex =>
                        match self.nibble_cursor {
                            NibbleCursor::LS => {
                                let byte = self.buffer[self.byte_cursor];
                                self.buffer[self.byte_cursor] = byte & 0b1111_0000;
                                self.nibble_cursor = NibbleCursor::MS;
                            }
                            NibbleCursor::MS =>
                                if self.byte_cursor >= self.buffer.len() && self.byte_cursor != 0 {
                                    self.byte_cursor -= 1;
                                    self.nibble_cursor = NibbleCursor::LS;
                                } else {
                                    match self.buffer.pop() {
                                        None => {
                                            self.nibble_cursor = NibbleCursor::MS;
                                        }
                                        Some(_) =>
                                            if self.byte_cursor != 0 {
                                                self.byte_cursor -= 1;
                                                self.nibble_cursor = NibbleCursor::LS;
                                            } else {
                                                self.nibble_cursor = NibbleCursor::MS;
                                            },
                                    }
                                },
                        },
                },
            Key::Char(ch) => {
                // FIXME non-ascii chars
                let ch = ch as u32;
                match self.mode {
                    SearchMode::Ascii =>
                        if ch <= 0xFF {
                            self.buffer.push(ch as u8);
                            self.byte_cursor += 1;
                            self.nibble_cursor = NibbleCursor::MS;
                        },
                    SearchMode::Hex => {
                        let nibble = match ch {
                            65...70 => {
                                // A ... F
                                Some((ch - 65 + 10) as u8)
                            }
                            97...102 => {
                                // a ... f
                                Some((ch - 97 + 10) as u8)
                            }
                            48...57 => {
                                // 0 ... 9
                                Some((ch - 48) as u8)
                            }
                            _ =>
                                None,
                        };

                        if let Some(nibble) = nibble {
                            let current_byte = if self.byte_cursor >= self.buffer.len() {
                                0
                            } else {
                                self.buffer[self.byte_cursor]
                            };

                            let new_byte = match self.nibble_cursor {
                                NibbleCursor::MS =>
                                    (current_byte & 0b0000_1111) | (nibble << 4),
                                NibbleCursor::LS =>
                                    (current_byte & 0b1111_0000) | nibble,
                            };

                            if self.byte_cursor >= self.buffer.len() {
                                self.buffer.push(new_byte);
                                self.nibble_cursor = NibbleCursor::LS;
                            } else {
                                self.buffer[self.byte_cursor] = new_byte;

                                match self.nibble_cursor {
                                    NibbleCursor::MS =>
                                        self.nibble_cursor = NibbleCursor::LS,
                                    NibbleCursor::LS => {
                                        self.nibble_cursor = NibbleCursor::MS;
                                        self.byte_cursor += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ =>
                {}
        }

        SearchRet::Continue
    }

    fn find_offsets(&self) -> Vec<usize> {
        let mut ret = Vec::new();

        let first_byte = self.buffer[0];

        // It seems like Vec API doesn't help us here. As a first
        // implementation, I do a O(n * k) search here.
        let mut byte_offset = 0;
        while byte_offset < self.contents.len() {
            let byte = unsafe { *self.contents.get_unchecked(byte_offset) };
            if byte == first_byte && try_match(&self.contents[byte_offset + 1..], &self.buffer[1..])
            {
                ret.push(byte_offset);
                byte_offset += self.buffer.len();
                continue;
            }

            byte_offset += 1;
        }

        // writeln!(&mut ::std::io::stderr(), "find_offsets: {:?}", ret);
        ret
    }
}

fn try_match(s1: &[u8], s2: &[u8]) -> bool {
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

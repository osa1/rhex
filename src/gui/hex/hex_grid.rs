use std::borrow::Borrow;
use std::cmp;
use std::ptr;

use std::io::prelude::*;

use gui::hex::HexGui;

use colors::Color;
use utils;

use ncurses as nc;

pub struct HexGrid<'grid> {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,

    data: &'grid Vec<u8>,
    path: &'grid str,

    cursor_x: i32,
    cursor_y: i32,
    scroll: i32,

    has_focus: bool,

    gui: *mut HexGui<'grid>,
}

impl<'grid> HexGrid<'grid> {
    pub fn new(width : i32, height : i32, pos_x : i32, pos_y : i32, data: &'grid Vec<u8>,
               path: &'grid str) -> HexGrid<'grid> {
        HexGrid {
            pos_x: pos_x,
            pos_y: pos_y,
            height: height,
            width: width,
            data: data,
            path: path,

            // Cursor positions are relative to the grid.
            // (i.e. they stay the same when grid is moved)
            cursor_x: 0,
            cursor_y: 0,
            scroll: 0,

            has_focus: false,

            gui: ptr::null_mut(),
        }
    }

    pub fn set_gui(&mut self, gui : *mut HexGui<'grid>) {
        self.gui = gui;
    }

    /// How many bytes we can show in a line?
    fn bytes_per_line(&self) -> i32 {
        let bytes = self.width / 3;

        // Can we fit one more column?
        let bytes =
            if self.width % 3 == 2 {
                bytes + 1
            } else {
                bytes
            };

        bytes
    }

    /// Effective width of a line (e.g. ignores extra trailing space that we
    /// can't utilize)
    fn cols_per_line(&self) -> i32 {
        self.bytes_per_line() * 3 - 1
    }

    /// How many lines needed to draw the entire file?
    fn total_lines_needed(&self) -> i32 {
        let len = self.data.len() as i32;
        let bpl = self.bytes_per_line();
        // round up
        (len + bpl - 1) / bpl
    }

    /// How many bytes do we render in last line? (this is usually different
    /// than self.width)
    fn last_line_bytes(&self) -> i32 {
        (self.data.len() % self.bytes_per_line() as usize) as i32
    }

    /// Unconditionally increment the Y position. Updates X position if there's
    /// not enough columns in the next line.
    ///
    /// This doesn't update anything other than X and Y positions.
    /// Only post-condition: post(self.pos_y) = self.pos_y + 1.
    fn move_next_line(&mut self) {
        let max_y = self.total_lines_needed() - 1;
        debug_assert!(self.cursor_y + 1 <= max_y);
        if self.cursor_y + 1 == max_y {
            let last_line_bytes = self.last_line_bytes();
            let last_line_cols  = (last_line_bytes - 1) * 3 + 2;
            if self.cursor_x >= last_line_cols {
                self.cursor_x = last_line_cols - 1;
            }
        }
        self.cursor_y += 1;
    }

    pub fn focus(&mut self, focus : bool) {
        self.has_focus = focus;
    }

    pub fn get_byte_idx(&self) -> i32 {
        self.cursor_y * self.bytes_per_line() + self.cursor_x / 3
    }

    pub fn get_column(&self) -> i32 {
        self.cursor_x
    }

    pub fn get_row(&self) -> i32 {
        self.cursor_y
    }

    pub fn get_scroll(&self) -> i32 {
        self.scroll
    }

    pub fn try_center_scroll(&mut self) {
        if self.cursor_y - self.height / 2 >= 0 {
            self.scroll = self.cursor_y - self.height / 2;
        }
    }

    pub fn keypressed(&mut self, key : i32) -> bool {
        if key == nc::KEY_UP || key == b'k' as i32 {
            if self.cursor_y > self.scroll + 2 && self.cursor_y > 0 {
                self.cursor_y -= 1;
            } else {
                if self.scroll > 0 {
                    self.scroll -= 1;
                    self.cursor_y -= 1;
                } else if self.cursor_y - 1 >= 0 {
                    self.cursor_y -= 1
                }
            }

            self.update_ascii_view();
            true
        }

        else if key == nc::KEY_DOWN || key == b'j' as i32 {
            // TODO: This assumes there's at least one line
            let max_y = self.total_lines_needed() - 1;

            if self.cursor_y < self.scroll + self.height - 3 && self.cursor_y < max_y {
                self.move_next_line();
            } else {
                // We want to scroll, but is there a text to show? Otherwise we
                // just move cursor down.
                if self.scroll + self.height <= max_y {
                    // We can scroll
                    self.scroll += 1;
                    // We move the cursor too, because it's not relative to the
                    // current scroll
                    self.cursor_y += 1;
                } else if self.cursor_y < max_y {
                    // We can't scroll but there's a line that we can move to
                    self.move_next_line();
                }
            }

            self.update_ascii_view();
            true
        }

        else if key == nc::KEY_LEFT || key == b'h' as i32 {
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
                if (self.cursor_x + 1) % 3 == 0 {
                    self.cursor_x -= 1;
                }
            }

            self.update_ascii_view();
            true
        }

        else if key == nc::KEY_RIGHT || key == b'l' as i32 {
            let next_on_blank =
                // add 1 to move to next column
                // add 1 to make the index 1-based
                (self.cursor_x + 1 + 1) % 3 == 0;

            let total_lines = self.total_lines_needed();
            let last_col_in_line =
                // FIXME: This won't work on empty files
                if self.cursor_y + 1 == total_lines {
                    // We're on the last line
                    (self.last_line_bytes() - 1) * 3 + 2
                } else {
                    self.cols_per_line()
                };

            let potential_next_col =
                if next_on_blank {
                    self.cursor_x + 2
                } else {
                    self.cursor_x + 1
                };

            if potential_next_col <= last_col_in_line {
                self.cursor_x = potential_next_col;
            }

            self.update_ascii_view();
            true
        }

        else if key == b'G' as i32 {
            self.move_cursor(self.data.len() as i32 - 1);
            true
        }

        else {
            false
        }
    }

    fn update_ascii_view(&self) {
        let gui : &mut HexGui = unsafe { &mut *self.gui };
        gui.get_ascii_view().move_cursor(self.get_byte_idx());
        gui.get_info_line().set_text(format!("{} - {}: {}",
                                             self.path,
                                             self.get_row(),
                                             self.get_column()).into_bytes().borrow());
    }

    pub fn draw(&self, hl: &Vec<usize>, hl_len: usize) {
        let cols = self.bytes_per_line();
        let rows = self.height;

        let mut hl_idx = 0;

        'outer:
        for row in self.scroll .. self.scroll + rows {
            for col in 0 .. cols {
                let byte_idx = (row * cols + col) as usize;
                if let Some(&byte) = self.data.get(byte_idx) {
                    let char1 : u8 = utils::hex_char(byte >> 4);
                    let char2 : u8 = utils::hex_char(byte & 0b00001111);

                    let attr_1 = col * 3     == self.cursor_x && row == self.cursor_y;
                    let attr_2 = col * 3 + 1 == self.cursor_x && row == self.cursor_y;
                    let color_attr =
                        if self.has_focus { Color::CursorFocus.attr() }
                        else { Color::CursorNoFocus.attr() };

                    while hl_idx < hl.len() && hl[hl_idx] + hl_len < byte_idx {
                        hl_idx += 1;
                    }

                    let hl_attr = {
                        if let Some(&hl_offset) = hl.get(hl_idx) {
                            if byte_idx >= hl_offset && byte_idx < hl_offset + hl_len {
                                // writeln!(&mut ::std::io::stderr(), "highlighting char: {}", byte as char);
                                Color::Highlight.attr()
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    };

                    if attr_1 {
                        nc::attron( nc::A_BOLD() | color_attr );
                    } else if hl_attr != 0 {
                        nc::attron( hl_attr );
                    }

                    nc::mvaddch( self.pos_y + row - self.scroll,
                                 self.pos_x + col * 3,     char1 as u64 );

                    if attr_1 {
                        nc::attroff( nc::A_BOLD() | color_attr );
                    } else if hl_attr != 0 {
                        nc::attroff( hl_attr );
                    }


                    if attr_2 {
                        nc::attron( nc::A_BOLD() | color_attr );
                    } else if hl_attr != 0 {
                        nc::attron( hl_attr );
                    }

                    nc::mvaddch( self.pos_y + row - self.scroll,
                                 self.pos_x + col * 3 + 1, char2 as u64 );

                    if attr_2 {
                        nc::attroff( nc::A_BOLD() | color_attr );
                    } else if hl_attr != 0 {
                        nc::attroff( hl_attr );
                    }

                    // When highlighting a word, paint the space between bytes too
                    let highlight = hl_attr != 0 && byte_idx + 1 < hl[hl_idx] + hl_len;

                    let space_col = self.pos_x + col * 3 + 2;
                    if highlight && space_col < self.width - 1 {
                        nc::attron( hl_attr );
                        nc::mvaddch( self.pos_y + row - self.scroll, space_col, b' ' as u64 );
                        nc::attroff( hl_attr );
                    }

                } else {
                    // Nothing to draw here, also we can break the loop
                    break 'outer;
                }
            }
        }
    }

    pub fn move_cursor(&mut self, byte_idx : i32) {
        let byte_idx = cmp::min((self.data.len() - 1) as i32, byte_idx);

        let bpl = self.bytes_per_line();
        self.cursor_y = byte_idx / bpl;
        self.cursor_x = (byte_idx % bpl) * 3;
        self.scroll   =
            if self.cursor_y < self.height { 0 }
            else { self.cursor_y - self.height / 2 };

        self.update_ascii_view();
    }
}

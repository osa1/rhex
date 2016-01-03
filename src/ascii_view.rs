use std::cmp;

use colors::Color;

use ncurses as nc;

pub struct AsciiView<'view> {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,

    data: &'view Vec<u8>,

    cursor_x: i32,
    cursor_y: i32,
    scroll: i32,

    has_focus: bool,
}

impl<'view> AsciiView<'view> {
    pub fn new(width : i32, height : i32, pos_x : i32, pos_y : i32, data: &Vec<u8>) -> AsciiView {
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

    pub fn draw(&self) {
        let rows = self.height;
        let cols = self.width;

        for row in self.scroll .. self.scroll + rows {
            for col in 0 .. cols {
                if row * cols + col < self.data.len() as i32 {
                    // Go unsafe, already checked the boundary
                    let byte = unsafe { *self.data.get_unchecked( (row * cols + col) as usize ) };

                    let ch =
                        if byte >= 32 && byte <= 126 {
                            byte
                        } else {
                            b'.'
                        };

                    let attr = self.cursor_x == col && self.cursor_y == row;
                    let color_attr =
                        if self.has_focus { Color::CursorFocus.attr() }
                        else { Color::CursorNoFocus.attr() };

                    if attr {
                        nc::attron( nc::A_BOLD() | color_attr );
                    }

                    nc::mvaddch( self.pos_y + row - self.scroll,
                                 self.pos_x + col,
                                 ch as u64 );

                    if attr {
                        nc::attroff( nc::A_BOLD() | color_attr );
                    }
                }
            }
        }
    }

    pub fn move_cursor(&mut self, byte_idx : i32) {
        let cursor_y = byte_idx / self.width;
        let cursor_x = byte_idx % self.width;

        if cursor_y > self.scroll + self.height - 3 {
            self.scroll = cursor_y - (self.height - 3);
        } else if cursor_y < self.scroll + 2 {
            self.scroll = cmp::max( cursor_y - 2, 0 );
        }

        self.cursor_y = cursor_y;
        self.cursor_x = cursor_x;
    }
}

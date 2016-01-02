use std::cmp;

use ncurses as nc;

// Guess what.. I think AsciiView is mostly HexGrid, with some overrided
// behavior.

use hex_grid::HexGrid;

pub struct AsciiView<'view>(HexGrid<'view>);

impl<'view> AsciiView<'view> {
    pub fn new(width : i32, height : i32, pos_x : i32, pos_y : i32, data: &Vec<u8>) -> AsciiView {
        AsciiView(HexGrid::new(width, height, pos_x, pos_y, data))
    }

    pub fn draw(&self) {
        let self_ = &self.0;

        let rows = self_.height;
        let cols = self_.width;

        for row in self_.scroll .. self_.scroll + rows {
            for col in 0 .. cols {
                if row * cols + col < self_.data.len() as i32 {
                    // Go unsafe, already checked the boundary
                    let byte = unsafe { *self_.data.get_unchecked( (row * cols + col) as usize ) };

                    let ch =
                        if byte >= 32 && byte <= 126 {
                            byte
                        } else {
                            b'.'
                        };

                    let attr = self_.cursor_x == col && self_.cursor_y == row;

                    if attr {
                        nc::attron( nc::A_BOLD() | nc::COLOR_PAIR(1) );
                    }

                    nc::mvaddch( self_.pos_y + row - self_.scroll,
                                 self_.pos_x + col,
                                 ch as u64 );

                    if attr {
                        nc::attroff( nc::A_BOLD() | nc::COLOR_PAIR(1) );
                    }
                }
            }
        }
    }

    pub fn move_cursor(&mut self, byte_idx : i32) {
        let cursor_y = byte_idx / self.0.width;
        let cursor_x = byte_idx % self.0.width;

        if cursor_y > self.0.scroll + self.0.height - 3 {
            self.0.scroll = cursor_y - (self.0.height - 3);
        } else if cursor_y < self.0.scroll + 2 {
            self.0.scroll = cmp::max( cursor_y - 2, 0 );
        }

        self.0.cursor_y = cursor_y;
        self.0.cursor_x = cursor_x;
    }
}

use std::borrow::Borrow;

use ncurses as nc;

pub struct HexGrid<'grid> {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,

    data: &'grid Vec<u8>,

    cursor_x: i32,
    cursor_y: i32,
    scroll: i32,
}

impl<'grid> HexGrid<'grid> {
    pub fn new(width : i32, height : i32, pos_x : i32, pos_y : i32, data: &Vec<u8>) -> HexGrid {
        HexGrid {
            pos_x: pos_x,
            pos_y: pos_y,
            height: height,
            width: width,
            data: data,

            // Cursor positions are relative to the grid.
            // (i.e. they stay the same when grid is moved)
            cursor_x: 0,
            cursor_y: 0,
            scroll: 0
        }
    }

    pub fn keypressed(&mut self, key : i32) {
        // TODO: Scroll

        if key == nc::KEY_UP {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
            }

        } else if key == nc::KEY_DOWN {
            if self.cursor_y < self.height {
                self.cursor_y += 1;
            }

        } else if key == nc::KEY_LEFT {
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
                if (self.cursor_x + 1) % 3 == 0 {
                    self.cursor_x -= 1;
                }
            }

        } else if key == nc::KEY_RIGHT {
            let on_blank =
                (self.cursor_x + self.pos_x) % 3 == 0;

            if on_blank {
                if self.cursor_x + self.pos_x + 2 < self.width {
                    self.cursor_x += 2;
                }
            } else {
                if self.cursor_x + self.pos_x + 1 < self.width {
                    self.cursor_x += 1;
                }
            }
        }

        nc::mv( self.cursor_y, self.cursor_x );
    }

    pub fn draw(&self) {
        let cols = self.width / 3;

        // Can we fit one more column?
        let cols =
            if self.width % 3 == 2 {
                cols + 1
            } else {
                cols
            };

        let rows = self.height;

        let contents_slice : &[u8] = self.data.borrow();
        let bytes : &[u8] = &contents_slice[0 .. (cols * rows) as usize];

        for row in 0 .. rows {
            for col in 0 .. cols {
                let byte = bytes[ (row * cols + col) as usize ];

                let char1 : u8 = hex_char(byte >> 4);
                let char2 : u8 = hex_char(byte & 0b00001111);

                let attr_1 = col * 3     == self.cursor_x && row == self.cursor_y;
                let attr_2 = col * 3 + 1 == self.cursor_x && row == self.cursor_y;

                if attr_1 {
                    nc::attron( nc::A_BOLD() | nc::COLOR_PAIR(1) );
                }

                nc::mvaddch( self.pos_y + row, self.pos_x + col * 3,     char1 as u64 );

                if attr_1 {
                    nc::attroff( nc::A_BOLD() | nc::COLOR_PAIR(1) );
                }


                if attr_2 {
                    nc::attron( nc::A_BOLD() | nc::COLOR_PAIR(1) );
                }

                nc::mvaddch( self.pos_y + row, self.pos_x + col * 3 + 1, char2 as u64 );

                if attr_2 {
                    nc::attroff( nc::A_BOLD() | nc::COLOR_PAIR(1) );
                }
            }
        }
    }

    pub fn narrow(&mut self) {
        if self.width > 3 {
            self.width -= 3;
        } else {
            self.width = 0;
        }
    }

    pub fn widen(&mut self) {
        self.width += 3;
    }
}

#[inline]
fn hex_char(byte : u8) -> u8 {
    if byte < 10 {
        48 + byte
    } else {
        97 + byte - 10
    }
}

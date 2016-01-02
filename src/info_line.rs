use std::borrow::Borrow;

use ncurses as nc;

pub struct InfoLine {
    pos_x: i32,
    pos_y: i32,
    width: i32,

    text: Vec<u8>,
}

impl InfoLine {
    pub fn new(width: i32, pos_x: i32, pos_y: i32, text: &[u8]) -> InfoLine {
        InfoLine {
            pos_x: pos_x,
            pos_y: pos_y,
            width: width,
            text: text.to_vec()
        }
    }

    pub fn set_text(&mut self, text: &[u8]) {
        self.text.clear();
        self.text.extend_from_slice(text);
    }

    pub fn draw(&self) {
        let slice : &[u8] = self.text.borrow();

        nc::attron( nc::COLOR_PAIR(2) );

        unsafe {
            nc::ll::mvaddnstr( self.pos_y, self.pos_x,
                               slice.as_ptr() as *const i8,
                               self.text.len() as i32 );
        }

        for x in (self.pos_x + self.text.len() as i32) .. (self.pos_x + self.width + 1) {
            unsafe { nc::ll::mvaddch( self.pos_y, x, b' ' as u64 ); }
        }

        nc::attroff( nc::COLOR_PAIR(2) );
    }
}

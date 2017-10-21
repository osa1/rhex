use utils::*;
use colors;

use termbox_simple::*;

pub struct InfoLine {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    text: String,
}

impl InfoLine {
    pub fn new(width: i32, pos_x: i32, pos_y: i32, text: String) -> InfoLine {
        InfoLine {
            pos_x: pos_x,
            pos_y: pos_y,
            width: width,
            text: text,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn draw(&self, tb: &mut Termbox) {
        let fg = colors::STATUS_BAR.fg;
        let bg = colors::STATUS_BAR.bg;

        for x in self.pos_x..=self.pos_x + self.width {
            tb.change_cell(x, self.pos_y, ' ', fg, bg);
        }

        print(tb, self.pos_x, self.pos_y, colors::STATUS_BAR, &self.text);
    }
}

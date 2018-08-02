use std::char;
use std::cmp;

use colors;
use utils::*;

use term_input::Key;
use termbox_simple::*;

/// Return value of the overlay. Returned by `keypressed()` method.
pub enum OverlayRet {
    /// User submitted the form.
    Ret(i32),

    /// For vi-like "go to beginning" (gg)
    GotoBeginning,

    /// User cancelled.
    Abort,

    /// Overlay still has focus.
    Continue,
}

pub struct GotoOverlay {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,
    input: String,
}

impl GotoOverlay {
    pub fn new(width: i32, height: i32, pos_x: i32, pos_y: i32) -> GotoOverlay {
        let width_ = cmp::min(width, 50);
        let height_ = cmp::min(height, 10);

        let pos_x = pos_x + (width - width_) / 2;
        let pos_y = pos_y + (height - height_) / 2;

        GotoOverlay {
            pos_x,
            pos_y,
            width: width_,
            height: height_,
            input: String::new(),
        }
    }

    pub fn draw(&self, tb: &mut Termbox) {
        draw_box(tb, self.pos_x, self.pos_y, self.width, self.height);
        print(
            tb,
            self.pos_x + 5,
            self.pos_y + 3,
            colors::DEFAULT,
            "Goto byte offset:",
        );
        print(tb, self.pos_x + 5, self.pos_y + 5, colors::DEFAULT, ">");
        print(
            tb,
            self.pos_x + 7,
            self.pos_y + 5,
            colors::DEFAULT,
            &self.input,
        );

        tb.change_cell(
            self.pos_x + 7 + self.input.len() as i32,
            self.pos_y + 5,
            ' ',
            colors::CURSOR_FOCUS.fg,
            colors::CURSOR_FOCUS.bg,
        );
    }

    pub fn keypressed(&mut self, key: Key) -> OverlayRet {
        match key {
            Key::Char(ch) if (ch >= '0' && ch <= '9') => {
                self.input.push(char::from_u32(ch as u32).unwrap());
                OverlayRet::Continue
            }
            Key::Char('g') =>
                OverlayRet::GotoBeginning,
            Key::Esc =>
                OverlayRet::Abort,
            Key::Backspace => {
                self.input.pop();
                OverlayRet::Continue
            }
            Key::Char('\r') =>
                if self.input.is_empty() {
                    OverlayRet::Abort
                } else {
                    OverlayRet::Ret(self.input.parse().unwrap())
                },
            _ =>
                OverlayRet::Continue,
        }
    }
}

use std::borrow::Borrow;
use std::char;
use std::cmp;

use colors::Color;

use ncurses as nc;

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
    pub win: nc::WINDOW,
    input: String,
}

impl Drop for GotoOverlay {
    fn drop(&mut self) {
        nc::delwin(self.win);
    }
}

impl GotoOverlay {
    pub fn new(width: i32, height: i32, pos_x: i32, pos_y: i32) -> GotoOverlay {
        let width_ = cmp::min(width, 50);
        let height_ = cmp::min(height, 10);

        let pos_x = pos_x + (width - width_) / 2;
        let pos_y = pos_y + (height - height_) / 2;

        GotoOverlay {
            win: nc::newwin(height_, width_, pos_y, pos_x),
            input: String::new(),
        }
    }

    pub fn draw(&self) {
        nc::wclear( self.win );

        nc::box_(self.win, 0, 0);

        nc::mvwaddstr( self.win, 3, 5, "Goto byte offset:" );
        nc::mvwaddstr( self.win, 5, 5, "> " );
        nc::mvwaddstr( self.win, 5, 7, self.input.borrow() );

        // Draw cursor
        nc::wattron( self.win, Color::CursorFocus.attr() );
        nc::mvwaddch( self.win, 5, 7 + self.input.len() as i32, b' ' as u64 );
        nc::wattroff( self.win, Color::CursorFocus.attr() );

        nc::wrefresh(self.win);
    }

    pub fn keypressed(&mut self, ch : i32) -> OverlayRet {
        if ch >= b'0' as i32 && ch <= b'9' as i32 {
            self.input.push(char::from_u32(ch as u32).unwrap());
            OverlayRet::Continue
        }

        else if ch == b'g' as i32 {
            OverlayRet::GotoBeginning
        }

        else if ch == 27 {
            // Is it escape or ALT + something?
            nc::nodelay( self.win, true );
            let next_ch = nc::wgetch( self.win );
            nc::nodelay( self.win, false );

            if next_ch == -1 {
                // It's escape, abort
                OverlayRet::Abort
            } else {
                // It's ALT + something, but we don't handle that
                OverlayRet::Continue
            }
        }

        else if ch == nc::KEY_BACKSPACE || ch == 127 { // backspace
            self.input.pop();
            OverlayRet::Continue
        }

        else if ch == 10 || ch == b'\n' as i32 {
            if self.input.len() == 0 {
                OverlayRet::Abort
            } else {
                OverlayRet::Ret( self.input.parse().unwrap() )
            }
        }

        else {
            OverlayRet::Continue
        }
    }
}

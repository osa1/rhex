use ncurses as nc;

pub enum Color {
    CursorNoFocus = 1,
    CursorFocus = 2,
    StatusBar = 3,
}

pub fn init_colors() {
    nc::start_color();
    nc::init_pair(Color::CursorNoFocus as i16, nc::COLOR_WHITE, nc::COLOR_YELLOW);
    nc::init_pair(Color::CursorFocus   as i16, nc::COLOR_WHITE, nc::COLOR_GREEN);
    nc::init_pair(Color::StatusBar     as i16, nc::COLOR_WHITE, nc::COLOR_RED);
}

impl Color {
    pub fn attr(self) -> nc::ll::chtype {
        nc::COLOR_PAIR(self as i16)
    }
}

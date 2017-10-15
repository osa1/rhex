use ncurses as nc;

pub enum Color {
    CursorNoFocus = 1,
    CursorFocus = 2,
    StatusBar = 3,
    Highlight = 4,

    FrameFocus = 5,
    FrameActive = 6,
}

pub fn init_colors() {
    nc::start_color();
    nc::init_pair(
        Color::CursorNoFocus as i16,
        nc::COLOR_WHITE,
        nc::COLOR_YELLOW,
    );
    nc::init_pair(Color::CursorFocus as i16, nc::COLOR_WHITE, nc::COLOR_GREEN);
    nc::init_pair(Color::StatusBar as i16, nc::COLOR_WHITE, nc::COLOR_RED);
    nc::init_pair(Color::Highlight as i16, nc::COLOR_BLACK, nc::COLOR_BLUE);
    nc::init_pair(Color::FrameFocus as i16, nc::COLOR_GREEN, nc::COLOR_BLACK);
    nc::init_pair(Color::FrameActive as i16, nc::COLOR_BLUE, nc::COLOR_BLACK);
}

impl Color {
    pub fn attr(self) -> nc::ll::chtype {
        nc::COLOR_PAIR(self as i16)
    }
}

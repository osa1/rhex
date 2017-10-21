use termbox_simple::*;

pub struct Style {
    pub fg: u16,
    pub bg: u16,
}

pub const DEFAULT: Style = Style {
    fg: TB_DEFAULT,
    bg: TB_DEFAULT,
};

pub const CURSOR_NO_FOCUS: Style = Style {
    fg: TB_WHITE,
    bg: TB_YELLOW,
};

pub const CURSOR_FOCUS: Style = Style {
    fg: TB_WHITE,
    bg: TB_GREEN,
};

pub const STATUS_BAR: Style = Style {
    fg: TB_WHITE,
    bg: TB_GREEN,
};

pub const HIGHLIGHT: Style = Style {
    fg: TB_BLACK,
    bg: TB_BLUE,
};

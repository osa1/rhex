mod ascii_view;
mod goto;
mod hex_grid;
mod info_line;
mod search;

use self::ascii_view::AsciiView;
use self::goto::{GotoOverlay, OverlayRet};
use self::hex_grid::HexGrid;
use self::info_line::InfoLine;
use self::search::{SearchOverlay, SearchRet};

use colors;
use utils::*;

use ncurses as nc;

/// GUI is the main thing that owns every widget. It's also responsible for
/// ncurses initialization and finalization.
pub struct HexGui<'gui> {
    hex_grid: HexGrid<'gui>,
    ascii_view: AsciiView<'gui>,
    info_line: InfoLine,
    overlay: Overlay<'gui>,
    contents: &'gui Vec<u8>,

    highlight: Vec<usize>,
    highlight_len: usize,
}

pub enum Overlay<'overlay> {
    NoOverlay,
    SearchOverlay(SearchOverlay<'overlay>),
    GotoOverlay(GotoOverlay),
}

// WARNING: Moving this after init() will cause a segfault. Not calling init()
// will cause a segfault.

impl<'gui> HexGui<'gui> {
    pub fn new(contents: &'gui Vec<u8>, path : &'gui str) -> HexGui<'gui> {
        nc::initscr();
        nc::keypad( nc::stdscr, true );
        nc::noecho();
        nc::curs_set( nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE );

        colors::init_colors();

        let mut scr_x = 0;
        let mut scr_y = 0;
        nc::getmaxyx(nc::stdscr, &mut scr_y, &mut scr_x);

        // Layout: We leave 2 spaces between hex view and ascii view. Every byte
        // takes 3 characters in hex view and 1 character in ascii view. So we
        // have this 3/1 ratio.

        let unit_column = scr_x / 4;

        let mut hex_grid = HexGrid::new( unit_column * 3, scr_y - 1, 0, 0, contents,
                                         path );

        let ascii_view = AsciiView::new( unit_column, scr_y - 1, unit_column * 3 + 1, 0,
                                         contents );


        let info_line = InfoLine::new( unit_column * 4, 0, scr_y - 1,
                                       format!("{} - 0: 0", path).as_bytes() );

        HexGui {
            hex_grid: hex_grid,
            ascii_view: ascii_view,
            info_line: info_line,
            overlay: Overlay::NoOverlay,
            contents: contents,

            highlight: Vec::new(),
            highlight_len: 0,
        }
    }

    pub fn init(&mut self) {
        let self_ptr = self as *mut HexGui;
        self.hex_grid.set_gui(self_ptr);
    }

    pub fn get_hex_grid(&mut self) -> &'gui mut HexGrid {
        &mut self.hex_grid
    }

    pub fn get_ascii_view(&mut self) -> &mut AsciiView<'gui> {
        &mut self.ascii_view
    }

    pub fn get_info_line(&mut self) -> &mut InfoLine {
        &mut self.info_line
    }

    pub fn draw(&self) {
        nc::clear();
        self.hex_grid.draw(&self.highlight, self.highlight_len);
        self.ascii_view.draw(&self.highlight, self.highlight_len);
        self.info_line.draw();
        nc::refresh();

        match self.overlay {
            Overlay::NoOverlay => {},
            Overlay::SearchOverlay(ref o) => o.draw(),
            Overlay::GotoOverlay(ref o) => o.draw(),
        }
    }

    pub fn mainloop(&mut self) {
        loop {
            let ch = self.get_char();

            if ch == b'q' as i32 {
                break;
            }

            let mut reset_overlay = false;
            match self.overlay {
                Overlay::NoOverlay => self.keypressed(ch),

                Overlay::GotoOverlay(ref mut o) => {
                    match o.keypressed(ch) {
                        OverlayRet::Ret(offset) => {
                            self.hex_grid.move_cursor(offset);
                            // self.overlay = Overlay::NoOverlay;
                            reset_overlay = true;
                        },
                        OverlayRet::GotoBeginning => {
                            self.hex_grid.move_cursor(0);
                            // self.overlay = Overlay::NoOverlay;
                            reset_overlay = true;
                        },
                        OverlayRet::Continue => {},
                        OverlayRet::Abort => {
                            // self.overlay = Overlay::NoOverlay;
                            reset_overlay = true;
                        },
                    }
                },

                Overlay::SearchOverlay(ref mut o) => {
                    match o.keypressed(ch) {
                        SearchRet::Highlight{ focus: f, all_bytes: bs, len: l } => {
                            self.highlight = bs;
                            self.highlight_len = l;
                            reset_overlay = true;
                        },
                        SearchRet::Abort => {
                            reset_overlay = true;
                        },
                        SearchRet::Continue => { /* nothing to do */ }
                    }
                }
            };

            if reset_overlay {
                self.overlay = Overlay::NoOverlay;
            }

            self.draw();
        }
    }

    fn keypressed(&mut self, ch : i32) {
        if ch == b'g' as i32 {
            self.mk_goto_overlay();
        }

        else if ch == b'/' as i32 {
            self.mk_search_overlay();
        }

        else if ch == b'z' as i32 {
            let next_ch = self.get_char();
            if next_ch == b'z' as i32 {
                self.hex_grid.try_center_scroll();
                self.ascii_view.set_scroll(self.hex_grid.get_scroll());
            } else {
                // ignore
            }
        }

        else if ch == b'n' as i32 {
            let hls = &self.highlight;
            let byte_idx = self.hex_grid.get_byte_idx() as usize;
            for &hl_offset in hls {
                if hl_offset > byte_idx {
                    self.hex_grid.move_cursor(hl_offset as i32);
                    return;
                }
            }
            // We couldn't jump to a match, start from the beginning
            if let Some(&hl_offset) = hls.get(0) {
                self.hex_grid.move_cursor(hl_offset as i32);
            }
        }

        else if ch == b'N' as i32 {
            let hls = &self.highlight;
            let byte_idx = self.hex_grid.get_byte_idx() as usize;
            for &hl_offset in hls.iter().rev() {
                if hl_offset < byte_idx {
                    self.hex_grid.move_cursor(hl_offset as i32);
                    return;
                }
            }
            // We couldn't jump to a match, start from the beginning
            if let Some(&hl_offset) = hls.get(hls.len() - 1) {
                self.hex_grid.move_cursor(hl_offset as i32);
            }
        }

        else {
            self.hex_grid.keypressed(ch);
        }
    }

    fn get_char(&self) -> i32 {
        match self.overlay {
            Overlay::NoOverlay => nc::getch(),
            Overlay::GotoOverlay(ref o) => o.get_char(),
            Overlay::SearchOverlay(ref o) => o.get_char(),
        }
    }

    fn mk_goto_overlay(&mut self) {
        let mut scr_x = 0;
        let mut scr_y = 0;
        nc::getmaxyx(nc::stdscr, &mut scr_y, &mut scr_x);

        self.overlay =
            Overlay::GotoOverlay(GotoOverlay::new( scr_x / 2, scr_y / 2, scr_x / 4, scr_y / 4 ));
    }

    fn mk_search_overlay(&mut self) {
        let mut scr_x = 0;
        let mut scr_y = 0;
        nc::getmaxyx(nc::stdscr, &mut scr_y, &mut scr_x);

        self.overlay =
            Overlay::SearchOverlay(
                SearchOverlay::new( scr_x / 2, scr_y / 2, scr_x / 4, scr_y / 4, &self.contents ));
    }
}

impl<'gui> Drop for HexGui<'gui> {
    fn drop(&mut self) {
        nc::endwin();
    }
}

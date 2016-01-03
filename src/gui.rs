use std::borrow::Borrow;

use ascii_view::AsciiView;
use colors;
use hex_grid::HexGrid;
use info_line::InfoLine;
use utils::*;

use ncurses as nc;

/// GUI is the main thing that owns every widget. It's also responsible for
/// ncurses initialization and finalization.
pub struct Gui<'gui> {
    hex_grid: Option<HexGrid<'gui>>,
    ascii_view: Option<AsciiView<'gui>>,
    info_line: Option<InfoLine>,
}

impl<'gui> Gui<'gui> {
    pub fn new() -> Gui<'gui> {
        nc::initscr();
        nc::keypad( nc::stdscr, true );
        nc::noecho();
        nc::curs_set( nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE );

        colors::init_colors();

        Gui { hex_grid: None, ascii_view: None, info_line: None }
    }

    pub fn init_widgets(&mut self, path : &'gui str, contents : &'gui Vec<u8>) {
        let mut scr_x = 0;
        let mut scr_y = 0;
        nc::getmaxyx(nc::stdscr, &mut scr_y, &mut scr_x);

        // Layout: We leave 2 spaces between hex view and ascii view. Every byte
        // takes 3 characters in hex view and 1 character in ascii view. So we
        // have this 3/1 ratio.

        let unit_column = scr_x / 4;

        self.hex_grid = Some(HexGrid::new( unit_column * 3, scr_y - 1, 0, 0, contents,
                                           path,
                                           self as *mut Gui ));

        self.ascii_view = Some(AsciiView::new( unit_column, scr_y - 1, unit_column * 3 + 1, 0,
                                               contents,
                                               self as *mut Gui ));

        self.info_line = Some(InfoLine::new( unit_column * 4, 0, scr_y - 1,
                                             format!("{} - 0: 0", path).as_bytes(),
                                             self as *mut Gui ));

        self.draw();
    }

    pub fn get_hex_grid(&mut self) -> &'gui mut Option<HexGrid> {
        &mut self.hex_grid
    }

    pub fn get_ascii_view(&mut self) -> &mut Option<AsciiView<'gui>> {
        &mut self.ascii_view
    }

    pub fn get_info_line(&mut self) -> &mut Option<InfoLine> {
        &mut self.info_line
    }

    pub fn draw(&mut self) {
        opt(&self.hex_grid, |g| g.draw());
        opt(&self.ascii_view, |g| g.draw());
        opt(&self.info_line, |g| g.draw());
    }

    pub fn mainloop(&mut self) {
        loop {
            let ch = nc::getch();

            if ch == b'q' as i32 {
                break;
            }

            opt_mut(&mut self.hex_grid, |g| { g.keypressed(ch); });

            nc::clear();
            self.draw();
            nc::refresh();
        }
    }
}

impl<'gui> Drop for Gui<'gui> {
    fn drop(&mut self) {
        nc::endwin();
    }
}
extern crate libc;
extern crate ncurses;

mod hex_grid;
mod ascii_view;

use std::env::args_os;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use ncurses as nc;

use hex_grid::HexGrid;
use ascii_view::AsciiView;

fn main() {
    let args : Vec<OsString> = args_os().collect();
    if args.len() != 2 {
        panic!("USAGE: rhex <file>");
    }

    let path = Path::new(&args[1]);
    let contents = match File::open(path) {
        Err(err) => panic!("Can't read file {:?}: {}", path, err),
        Ok(mut file) => {
            let mut ret = Vec::new();
            file.read_to_end(&mut ret).unwrap();
            ret
        }
    };

    mainloop(&contents);
}

struct NCurses;

impl NCurses {
    pub fn new() -> NCurses {
        nc::initscr();
        NCurses
    }
}

impl Drop for NCurses {
    fn drop(&mut self) {
        nc::endwin();
    }
}

fn mainloop(contents : &Vec<u8>) {
    let _nc = NCurses::new();
    nc::keypad(nc::stdscr, true);
    nc::noecho();
    nc::curs_set( nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE );

    nc::start_color();
    nc::init_pair(1, nc::COLOR_WHITE, nc::COLOR_GREEN);

    let mut scr_x = 0;
    let mut scr_y = 0;
    nc::getmaxyx(nc::stdscr, &mut scr_y, &mut scr_x);

    // Layout: We leave 2 spaces between hex view and ascii view. Every byte
    // takes 3 characters in hex view and 1 character in ascii view. So we have
    // this 3/1 ratio.

    let unit_column = scr_x / 4;

    let mut grid = HexGrid::new( unit_column * 3, scr_y, 0, 0, contents );
    grid.draw();

    let mut ascii_view = AsciiView::new( unit_column, scr_y,
                                         unit_column * 3 + 1, 0,
                                         contents );
    ascii_view.draw();

    nc::refresh();

    loop {
        let ch = nc::getch();

        if ch == 27 {
            break;

        } else if ch == b'e' as i32 {
            grid.widen();
            nc::clear();
            grid.draw();
            ascii_view.draw();
            nc::refresh();

        } else if ch == b'q' as i32 {
            grid.narrow();
            nc::clear();
            grid.draw();
            ascii_view.draw();
            nc::refresh();

        } else {
            if grid.keypressed(ch) {
                ascii_view.move_cursor(grid.get_byte_idx());
                nc::clear();
                grid.draw();
                ascii_view.draw();
                nc::refresh();
            }
        }
    }
}

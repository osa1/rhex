extern crate libc;
extern crate ncurses;

mod hex_grid;

use std::env::args_os;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use ncurses::*;

use hex_grid::HexGrid;

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

fn mainloop(contents : &Vec<u8>) {
    initscr();
    keypad(stdscr, true);
    // timeout(-1);
    noecho();
    curs_set( CURSOR_VISIBILITY::CURSOR_INVISIBLE );

    start_color();
    init_pair(1, COLOR_RED, COLOR_BLACK);

    let mut scr_x = 0;
    let mut scr_y = 0;
    getmaxyx(stdscr, &mut scr_y, &mut scr_x);

    let scr_x =
        if scr_x % 3 == 1 {
            // We can't fit one more column, just act like we have just enough
            // space
            scr_x - 1
        } else {
            // mod == 0: We barely fit columns
            // mod == 2: Draw functions are smart enough to not draw space after
            //           last column, so it's OK
            scr_x
        };

    let mut grid = HexGrid::new( scr_x - 5, scr_y - 5, 2, 2, contents );
    grid.draw();
    refresh();

    loop {
        let ch = getch();

        if ch == 27 {
            break;

        } else if ch == 101 { // e
            grid.widen();
            clear();
            grid.draw();
            refresh();

        } else if ch == 113 { // q
            grid.narrow();
            clear();
            grid.draw();
            refresh();

        } else {
            grid.keypressed(ch);
            grid.draw();
            refresh();
        }
    }

    endwin();
}

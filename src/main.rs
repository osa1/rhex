#![feature(alloc_system)]

extern crate alloc_system;

extern crate libc;
extern crate ncurses;

mod colors;
mod gui;
mod utils;

use std::env::args_os;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gui::Gui;

use ncurses as nc;

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

    nc::initscr();
    nc::keypad(nc::stdscr(), true);
    nc::noecho();
    nc::curs_set( nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE );

    colors::init_colors();

    let mut scr_x = 0;
    let mut scr_y = 0;
    nc::getmaxyx(nc::stdscr(), &mut scr_y, &mut scr_x);

    let mut gui = Gui::new_hex_gui(&contents, path.to_str().unwrap(),
                                   scr_x, scr_y, 0, 0);
    gui.mainloop();
}

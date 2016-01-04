extern crate libc;
extern crate ncurses;

mod ascii_view;
mod colors;
mod goto;
mod gui;
mod hex_grid;
mod info_line;
mod utils;

use std::borrow::Borrow;
use std::env::args_os;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use ascii_view::AsciiView;
use gui::Gui;
use hex_grid::HexGrid;
use info_line::InfoLine;

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

    let mut gui = Gui::new();
    gui.init_widgets(path.to_str().unwrap(), &contents);

    gui.mainloop();
}

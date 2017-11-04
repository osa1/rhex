#![feature(alloc_system)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(inclusive_range_syntax)]

extern crate alloc_system;

#[global_allocator]
static ALLOC: alloc_system::System = alloc_system::System;

extern crate libc;
extern crate nix;
extern crate term_input;
extern crate termbox_simple;

mod colors;
mod gui;
mod utils;

use std::env::args_os;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gui::Gui;

use termbox_simple::*;

fn main() {
    let args: Vec<OsString> = args_os().collect();
    if args.len() != 2 {
        panic!("USAGE: rhex <file>");
    }

    let path = Path::new(&args[1]);
    let contents = match File::open(path) {
        Err(err) =>
            panic!("Can't read file {:?}: {}", path, err),
        Ok(mut file) => {
            let mut ret = Vec::new();
            file.read_to_end(&mut ret).unwrap();
            ret
        }
    };

    let mut tb = Termbox::init().unwrap();
    tb.set_output_mode(OutputMode::Output256);
    tb.set_clear_attributes(TB_DEFAULT, TB_DEFAULT);

    let scr_x = tb.width();
    let scr_y = tb.height();

    let mut gui = Gui::new_hex_gui(tb, &contents, path.to_str().unwrap(), scr_x, scr_y);
    gui.mainloop();
}

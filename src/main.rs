#![feature(alloc_system)]
#![feature(time2)]

extern crate alloc_system;

extern crate libc;
extern crate ncurses;

mod colors;
mod gui;
mod parser;
mod utils;

use std::borrow::Borrow;
use std::env::args_os;
use std::ffi::CString;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gui::Gui;
use parser::elf;

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
    nc::keypad( nc::stdscr, true );
    nc::noecho();
    nc::curs_set( nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE );

    colors::init_colors();

    let mut scr_x = 0;
    let mut scr_y = 0;
    nc::getmaxyx(nc::stdscr, &mut scr_y, &mut scr_x);

    let mut gui = Gui::new_hex_gui(&contents, path.to_str().unwrap(),
                                   scr_x, scr_y, 0, 0);
    if let elf::ParseResult::ParseOK(elf_header) = elf::parse_elf_header(path) {
        let program_headers = elf::parse_program_headers(&elf_header, &contents);
        let section_headers = elf::parse_section_headers(&elf_header, &contents);
        gui.init_elf_gui(elf_header, program_headers, section_headers);
    }

    gui.mainloop();

    /*
    let path = Path::new("/home/omer/bin/ag");
    let mut contents = Vec::new();
    let header = match File::open(path) {
        Err(err) => panic!(),
        Ok(mut file) => {
            file.read_to_end(&mut contents);
            match parser::elf::parse_elf_header_(contents.borrow()) {
                parser::elf::ParseResult::ParseOK(elf) => elf,
                _ => panic!(),
            }
        }
    };

    println!("{:?}", header);

    let section_headers = parser::elf::parse_section_headers(&header, contents.borrow());
    // for header in section_headers.iter() {
    //     println!("{:?}", header);
    // }
    // println!("{:?}", section_headers);

    if let parser::elf::SectionHeader::SectionHeader64(ref hdr) =
        section_headers[ header.shstrndx as usize ] {

            println!("{:?}", hdr);

            let str_offset = hdr.offset as usize;
            let str_size   = hdr.size as usize;

            let strings = &contents[ str_offset .. str_offset + str_size + 1 ];

            let parsed : Vec<CString> =
                parser::elf::read_string_table(strings)
                    .into_iter()
                    .map(|s| CString::new(s).unwrap())
                    .collect();

            println!("{:?}", parsed);
    };

    println!("--------------------------------");
    let pgm_headers = parser::elf::parse_program_headers(&header, contents.borrow());
    for header in pgm_headers.iter() {
        println!("{:?}", header);
    }
    */
}

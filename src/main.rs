extern crate libc;
extern crate ncurses;

mod ascii_view;
mod colors;
mod goto;
mod gui;
mod hex_grid;
mod info_line;
mod parser;
mod search;
mod utils;

use std::borrow::Borrow;
use std::env::args_os;
use std::ffi::CString;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gui::Gui;

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

    let mut gui = Gui::new(&contents, path.to_str().unwrap());
    gui.init();
    gui.draw();
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

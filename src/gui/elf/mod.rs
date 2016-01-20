mod fields;
mod program_header;

use std::borrow::Borrow;

use colors::Color;
use gui::GuiRet;
use parser::elf;
use utils::draw_box;

use ncurses as nc;

pub struct ElfGui {
    elf_header : elf::ELFHeader,
    elf_header_fields : Vec<Box<fields::Field>>,

    section_headers : Vec<elf::SectionHeader>,

    program_headers : Vec<elf::ProgramHeader>,
    program_header_fields : Vec<program_header::ProgramHeader>,

    // layout related stuff
    width : i32,
    height : i32,
    pos_x : i32,
    pos_y : i32,

    scroll : i32,

    cursor : Cursor,
}

#[derive(Clone, Copy, PartialEq)]
enum Cursor {
    ElfHeader(usize),

    ProgramHeader {
        phdr_idx : usize,
        phdr_field : usize
    },

    SectionHeader {
        shdr_idx : usize,
        shdr_field : usize
    },
}

impl ElfGui {
    pub fn new(elf_header: elf::ELFHeader,
               section_headers: Vec<elf::SectionHeader>,
               program_headers: Vec<elf::ProgramHeader>,
               width: i32, height: i32, pos_x: i32, pos_y: i32) -> ElfGui {
        ElfGui {
            // OMG this is ridiculous. This line needs to come first because in
            // the next line we move the header. It turns out rustc can't
            // reorder these for me.
            elf_header_fields: fields::mk_elf_hdr_fields(&elf_header),
            elf_header: elf_header,

            section_headers: section_headers,

            program_header_fields: program_header::mk_pgm_hdr_fields(&program_headers),
            program_headers: program_headers,

            width: width,
            height: height,
            pos_x: pos_x,
            pos_y: pos_y,

            scroll: 0,
            cursor: Cursor::ElfHeader(0),
        }
    }

    pub fn mainloop(&mut self) -> GuiRet {
        // We don't have timed events, set ncurses to blocking read mode
        nc::timeout(-1);

        loop {
            self.draw();

            let ch = self.get_char();

            if ch == b'q' as i32 {
                return GuiRet::Break;
            } else if ch == b'\t' as i32 {
                return GuiRet::Switch;
            } else {
                self.keypressed(ch);
            }
        }
    }

    pub fn draw(&self) {
        self.draw_elf_header();

        // Draw program headers

        let header_height = self.elf_header_height();

        let box_x = self.pos_x + 1;
        let box_y = self.pos_y + header_height + 1;

        let box_width = self.width - 2;
        let box_height = self.program_header_fields[0].get_height();

        let header_title = "Program header";

        for (hdr_idx, pgm_hdr) in self.program_header_fields.iter().enumerate() {
            let y = box_y + (box_height + 1) * (hdr_idx as i32);

            draw_box(box_x, y, box_width, box_height, Some(header_title.borrow()));

            // Draw program header fields
            for (field_idx, field) in pgm_hdr.fields.iter().enumerate() {
                field.draw(box_x + 1, y + 1 + field_idx as i32, box_width - 2, box_height - 2, false);
            }
        }
    }

    pub fn keypressed(&mut self, ch : i32) {
        let mut next_cursor = self.cursor;
        if ch == nc::KEY_UP || ch == b'k' as i32 {
            match self.cursor {
                Cursor::ElfHeader(elf_header_cursor) => {
                    let field = &self.elf_header_fields[elf_header_cursor];
                    match field.prev() {
                        fields::FieldRet::Prev => {},
                        fields::FieldRet::Field(field_idx) => {
                            next_cursor = Cursor::ElfHeader(field_idx);
                        },
                        fields::FieldRet::Next => {
                            // next_cursor = Cursor::ProgramHeader {
                            //     phdr_idx: 0,
                            //     phdr_field: 0,
                            // };
                        }
                    }
                },
                Cursor::ProgramHeader { phdr_idx, phdr_field } => {

                },
                Cursor::SectionHeader { shdr_idx, shdr_field } => {

                },
            }
        }

        else if ch == nc::KEY_DOWN || ch == b'j' as i32 {
            match self.cursor {
                Cursor::ElfHeader(elf_header_cursor) => {
                    let field = &self.elf_header_fields[elf_header_cursor];
                    match field.next() {
                        fields::FieldRet::Prev => {},
                        fields::FieldRet::Field(field_idx) => {
                            next_cursor = Cursor::ElfHeader(field_idx);
                        },
                        fields::FieldRet::Next => {
                            // next_cursor = Cursor::ProgramHeader {
                            //     phdr_idx: 0,
                            //     phdr_field: 0,
                            // };
                        }
                    }
                },
                Cursor::ProgramHeader { phdr_idx, phdr_field } => {

                },
                Cursor::SectionHeader { shdr_idx, shdr_field } => {

                },
            }
        }

        self.cursor = next_cursor;
    }

    fn draw_elf_header(&self) {
        // for now assume each field takes one row
        for (field_idx, field) in self.elf_header_fields.iter().enumerate() {
            let focus = match self.cursor {
                Cursor::ElfHeader(idx) => field.get_idx() == idx,
                _ => false,
            };

            field.draw(self.pos_x + 1,
                       self.pos_y + (field_idx as i32) + 1,
                       self.width,
                       self.height,
                       focus);
        }
    }

    fn elf_header_height(&self) -> i32 {
        self.elf_header_fields.len() as i32
    }

    fn get_char(&self) -> i32 {
        nc::getch()
    }
}

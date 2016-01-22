mod field;
mod program_header;

use std::borrow::Borrow;

use colors::Color;
use gui::GuiRet;
use parser::elf;
use utils::draw_box;

use ncurses as nc;

pub struct ElfGui {
    elf_header : elf::ELFHeader,
    elf_header_fields : Vec<Box<field::Field>>,

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
        phdr_focused : bool,
    },

    SectionHeader {
        shdr_idx : usize,
        shdr_focused : bool,
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
            elf_header_fields: field::mk_elf_hdr_fields(&elf_header),
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

        for (hdr_idx, pgm_hdr) in self.program_header_fields.iter().enumerate() {
            let y = box_y + (box_height + 1) * (hdr_idx as i32);

            let highlight = match self.cursor {
                // TODO: How to wildcard unused fields?
                Cursor::ProgramHeader { phdr_idx, phdr_focused } => phdr_idx == hdr_idx,
                _ => false,
            };

            pgm_hdr.draw(box_x, y, box_width, box_height, highlight);
        }
    }

    pub fn keypressed(&mut self, ch : i32) {
        let mut next_cursor = self.cursor;
        if ch == nc::KEY_UP || ch == b'k' as i32 {
            match self.cursor {
                Cursor::ElfHeader(elf_header_cursor) => {
                    let field = &self.elf_header_fields[elf_header_cursor];
                    match field.prev() {
                        field::FieldRet::Prev => {},
                        field::FieldRet::Field(field_idx) => {
                            next_cursor = Cursor::ElfHeader(field_idx);
                        },
                        field::FieldRet::Next => {
                            next_cursor = Cursor::ProgramHeader {
                                phdr_idx: 0,
                                phdr_focused: false,
                            };
                        },
                    }
                },
                Cursor::ProgramHeader { phdr_idx, phdr_focused } => {
                    if !phdr_focused {
                        if phdr_idx > 0 {
                            next_cursor = Cursor::ProgramHeader {
                                phdr_idx: phdr_idx - 1,
                                phdr_focused: false,
                            }
                        } else {
                            next_cursor = Cursor::ElfHeader(self.elf_header_fields.len() - 1);
                        }
                    }
                },
                Cursor::SectionHeader { shdr_idx, shdr_focused } => {

                },
            }
        }

        else if ch == nc::KEY_DOWN || ch == b'j' as i32 {
            match self.cursor {
                Cursor::ElfHeader(elf_header_cursor) => {
                    let field = &self.elf_header_fields[elf_header_cursor];
                    match field.next() {
                        field::FieldRet::Prev => {},
                        field::FieldRet::Field(field_idx) => {
                            next_cursor = Cursor::ElfHeader(field_idx);
                        },
                        field::FieldRet::Next => {
                            next_cursor = Cursor::ProgramHeader {
                                phdr_idx: 0,
                                phdr_focused: false,
                            };
                        }
                    }
                },
                Cursor::ProgramHeader { phdr_idx, phdr_focused } => {
                    if !phdr_focused && phdr_idx < self.program_headers.len() - 1 {
                        next_cursor = Cursor::ProgramHeader {
                            phdr_idx: phdr_idx + 1,
                            phdr_focused: false,
                        }
                    }
                },
                Cursor::SectionHeader { shdr_idx, shdr_focused } => {

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

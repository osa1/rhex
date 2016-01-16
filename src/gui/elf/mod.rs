mod fields;

use colors::Color;
use gui::GuiRet;
use parser::elf;

use ncurses as nc;

pub struct ElfGui {
    elf_header : elf::ELFHeader,
    elf_header_fields : Vec<Box<fields::Field>>,

    section_headers: Vec<elf::SectionHeader>,

    program_headers: Vec<elf::ProgramHeader>,

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
        // let top    = self.scroll;
        // let bottom = top + self.height;

        self.draw_elf_header();
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
                            next_cursor = Cursor::ProgramHeader {
                                phdr_idx: 0,
                                phdr_field: 0,
                            };
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
                            next_cursor = Cursor::ProgramHeader {
                                phdr_idx: 0,
                                phdr_field: 0,
                            };
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

    fn get_char(&self) -> i32 {
        nc::getch()
    }
}

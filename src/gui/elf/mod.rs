mod field;
mod program_header;

use std::borrow::Borrow;

use colors::Color;
use gui::GuiRet;
use parser::elf;
use self::program_header::{ProgramHeader, ProgramHeaderRet};
use utils::draw_box;

use ncurses as nc;

pub struct ElfGui<'gui> {
    elf_header : elf::ELFHeader,
    elf_header_fields : Vec<Box<field::Field>>,

    section_headers : Vec<elf::SectionHeader<'gui>>,

    program_headers : Vec<elf::ProgramHeader<'gui>>,
    program_header_fields : Vec<ProgramHeader>,

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

impl<'gui> ElfGui<'gui> {
    pub fn new(elf_header: elf::ELFHeader,
               section_headers: Vec<elf::SectionHeader<'gui>>,
               program_headers: Vec<elf::ProgramHeader<'gui>>,
               width: i32, height: i32, pos_x: i32, pos_y: i32) -> ElfGui<'gui> {
        ElfGui::<'gui> {
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

    pub fn keypressed(&mut self, key : i32) {
        match self.cursor {
            Cursor::ElfHeader(idx) => {
                self.handle_elf_header_keypress(key, idx);
            },
            Cursor::ProgramHeader { phdr_idx, phdr_focused } => {
                self.handle_phdr_keypress(key, phdr_idx, phdr_focused);
            },
            Cursor::SectionHeader { shdr_idx, shdr_focused } => {},
        }
    }

    fn handle_elf_header_keypress(&mut self, key : i32, idx : usize) {
        if key == nc::KEY_UP || key == b'k' as i32 {
            if idx > 0 {
                let idx = idx - 1;
                self.cursor = Cursor::ElfHeader(idx);
                self.elf_hdr_scroll_up(idx);
            }
        }

        else if key == nc::KEY_DOWN || key == b'j' as i32 {
            if idx < self.elf_header_fields.len() - 1 {
                self.cursor = Cursor::ElfHeader(idx + 1);
            } else if self.program_header_fields.len() > 0 {
                self.cursor = Cursor::ProgramHeader {
                    phdr_idx: 0,
                    phdr_focused: false,
                }
            }
        }
    }

    fn handle_phdr_keypress(&mut self, key : i32, idx : usize, focused : bool) {
        if focused {
            match self.program_header_fields[idx].keypressed(key) {
                ProgramHeaderRet::LostFocus => {
                    self.cursor = Cursor::ProgramHeader {
                        phdr_idx: idx,
                        phdr_focused: false,
                    };
                }
                ProgramHeaderRet::KeyHandled => {},
                ProgramHeaderRet::KeyIgnored => {},
            }
        } else {
            if key == nc::KEY_UP || key == b'k' as i32 {
                if idx > 0 {
                    let idx = idx - 1;
                    self.cursor = Cursor::ProgramHeader {
                        phdr_idx: idx,
                        phdr_focused: false,
                    };
                    self.phdr_scroll_up(idx);
                } else {
                    let idx = self.elf_header_fields.len() - 1;
                    self.cursor = Cursor::ElfHeader(idx);
                    self.elf_hdr_scroll_up(idx);
                }
            }

            else if key == nc::KEY_DOWN || key == b'j' as i32 {
                if idx < self.program_headers.len() - 1 {
                    let idx = idx + 1;
                    self.cursor = Cursor::ProgramHeader {
                        phdr_idx: idx,
                        phdr_focused: false,
                    };
                    self.phdr_scroll_down(idx);
                }
            }

            else if key == 10 || key == b'\n' as i32 {
                self.program_header_fields[idx].focus();
                self.cursor = Cursor::ProgramHeader {
                    phdr_idx: idx,
                    phdr_focused: true,
                };
            }
        }
    }

    pub fn draw(&self) {
        nc::clear();
        self.draw_elf_header();

        // Draw program headers

        let header_height = self.elf_header_height();

        let box_x = self.pos_x + 1;
        let box_y = self.pos_y + header_height;

        let box_width = self.width - 2;
        let box_height = self.program_header_fields[0].get_height();

        for (hdr_idx, pgm_hdr) in self.program_header_fields.iter().enumerate() {
            let y = box_y + (box_height + 2) * (hdr_idx as i32);

            let highlight = match self.cursor {
                // TODO: How to wildcard unused fields?
                Cursor::ProgramHeader { phdr_idx, phdr_focused } => phdr_idx == hdr_idx,
                _ => false,
            };

            pgm_hdr.draw(box_x, y - self.scroll, box_width, box_height + 1, highlight);
        }
    }

    fn draw_elf_header(&self) {
        // for now assume each field takes one row
        for (field_idx, field) in self.elf_header_fields.iter().enumerate() {
            let focus = match self.cursor {
                Cursor::ElfHeader(idx) => field_idx == idx,
                _ => false,
            };

            field.draw(self.pos_x + 1,
                       self.pos_y + (field_idx as i32) - self.scroll,
                       self.width,
                       self.height,
                       focus);
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Scrolling
    ////////////////////////////////////////////////////////////////////////////

    fn elf_hdr_scroll_up(&mut self, idx : usize) {
        if self.scroll > idx as i32 {
            self.scroll = idx as i32;
        }
    }

    fn phdr_scroll_up(&mut self, idx : usize) {
        let header_height = self.elf_header_height();
        let mut box_top = header_height;
        for i in 0 .. idx {
            box_top += self.program_header_fields[idx].get_height() + 2;
        }

        if box_top < self.scroll {
            self.scroll = box_top;
        }
    }

    fn phdr_scroll_down(&mut self, idx : usize) {
        // TODO: Move the frame to ProgramHeader, remove +2s.
        let header_height = self.elf_header_height();
        let mut box_bottom =
            header_height + self.program_header_fields[idx].get_height() + 2;
        for i in 0 .. idx {
            box_bottom += self.program_header_fields[i].get_height() + 2;
        }

        if box_bottom > self.height + self.scroll {
            self.scroll += box_bottom - self.scroll - self.height;
        }
    }

    ////////////////////////////////////////////////////////////////////////////

    fn elf_header_height(&self) -> i32 {
        self.elf_header_fields.len() as i32
    }

    fn get_char(&self) -> i32 {
        nc::getch()
    }
}

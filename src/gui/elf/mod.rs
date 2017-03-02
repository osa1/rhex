mod field;
mod program_header;
mod section_header;
mod widget;

use std::borrow::Borrow;

use colors::Color;
use gui::GuiRet;
use parser::elf;
use self::program_header::{ProgramHeader};
use self::section_header::{SectionHeader};
use self::widget::{Widget, WidgetRet};
use utils::draw_box;

use ncurses as nc;

pub struct ElfGui<'gui> {
    elf_header : elf::ELFHeader,
    section_headers : Vec<elf::SectionHeader<'gui>>,
    program_headers : Vec<elf::ProgramHeader<'gui>>,
    string_table : Option<elf::StringTable>,

    fields : Vec<Box<Widget>>,

    // layout related stuff
    width : i32,
    height : i32,
    pos_x : i32,
    pos_y : i32,

    scroll : i32,

    cursor : Cursor,
}

#[derive(Clone, Copy, PartialEq)]
struct Cursor {
    idx : usize,
    focused : bool,
}

impl<'gui> ElfGui<'gui> {
    pub fn new(elf_header: elf::ELFHeader,
               section_headers: Vec<elf::SectionHeader<'gui>>,
               program_headers: Vec<elf::ProgramHeader<'gui>>,
               string_table: Option<elf::StringTable>,
               width: i32, height: i32, pos_x: i32, pos_y: i32) -> ElfGui<'gui> {

        let mut fields = field::mk_elf_hdr_fields(&elf_header);
        fields.append(&mut program_header::mk_pgm_hdr_fields(&program_headers));
        fields.append(&mut section_header::mk_sec_hdr_fields(&section_headers, &string_table));

        ElfGui::<'gui> {
            elf_header: elf_header,
            section_headers: section_headers,
            program_headers: program_headers,
            string_table: string_table,
            fields: fields,

            width: width,
            height: height,
            pos_x: pos_x,
            pos_y: pos_y,

            scroll: 0,
            cursor: Cursor { idx: 0, focused : false },
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
        if self.cursor.focused {
            match self.fields[self.cursor.idx].keypressed(key) {
                WidgetRet::LostFocus => {
                    self.cursor.focused = false;
                },
                WidgetRet::KeyHandled => {},
                WidgetRet::KeyIgnored => {},
            }
        } else {
            if key == nc::KEY_UP || key == b'k' as i32 {
                if self.cursor.idx > 0 {
                    self.cursor.idx -= 1;
                    self.scroll_up();
                }
            }

            else if key == nc::KEY_DOWN || key == b'j' as i32 {
                if self.cursor.idx < self.fields.len() - 1 {
                    self.cursor.idx += 1;
                    self.scroll_down();
                }
            }

            else if key == 10 || key == b'\n' as i32 {
                if self.fields[self.cursor.idx].focus() {
                    self.cursor.focused = true;
                }
            }
        }
    }

    pub fn draw(&self) {
        if self.fields.len() == 0 {
            return;
        }

        nc::clear();

        let box_x = self.pos_x + 1;
        let mut box_y = self.pos_y;

        let box_width = self.width - 2;

        for (hdr_idx, hdr) in self.fields.iter().enumerate() {
            if hdr_idx > 0 {
                box_y += self.fields[hdr_idx - 1].get_height();
            }

            let highlight = self.cursor.idx == hdr_idx;

            hdr.draw(box_x, box_y - self.scroll, box_width, hdr.get_height(), highlight);
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Scrolling
    ////////////////////////////////////////////////////////////////////////////

    fn scroll_up(&mut self) {
        let mut box_top = 0;
        for i in 0 .. self.cursor.idx {
            box_top += self.fields[i].get_height();
        }

        if box_top < self.scroll {
            self.scroll = box_top;
        }
    }

    fn scroll_down(&mut self) {
        let mut box_bottom = 0;
        for i in 0 .. self.cursor.idx + 1 {
            box_bottom += self.fields[i].get_height();
        }

        // TODO: We need to make sure top of the widget will be in bounds
        if box_bottom > self.height + self.scroll {
            self.scroll += box_bottom - self.scroll - self.height;
        }
    }

    ////////////////////////////////////////////////////////////////////////////

    fn get_char(&self) -> i32 {
        nc::getch()
    }
}

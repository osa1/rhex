use parser::elf;

use gui::GuiRet;

use ncurses as nc;

pub struct ElfGui<'gui> {
    elf_header : &'gui elf::ELFHeader,
    section_headers: &'gui Vec<elf::SectionHeader>,
    program_headers: &'gui Vec<elf::ProgramHeader>,

    // layout related stuff
    width : i32,
    height : i32,
    pos_x : i32,
    pos_y : i32,

    scroll : i32,

    cursor : Cursor,
}

enum Cursor {
    ElfHeader,
    ProgramHeader(usize, ProgramHeaderCursor),
    SectionHeader(usize, SectionHeaderCursor),
}

enum ProgramHeaderCursor {
    Ty, Offset, Vaddr, Paddr, Filesz, Memsz, Flags, Align
}

enum SectionHeaderCursor {
    Name, Ty, Flags, Addr, Offset, Size, Link, Info, Addralign, Entsize
}

impl<'gui> ElfGui<'gui> {
    pub fn new(elf_header: &'gui elf::ELFHeader,
               section_headers: &'gui Vec<elf::SectionHeader>,
               program_headers: &'gui Vec<elf::ProgramHeader>,
               width: i32, height: i32, pos_x: i32, pos_y: i32) -> ElfGui<'gui> {
        ElfGui {
            elf_header: elf_header,
            section_headers: section_headers,
            program_headers: program_headers,
            width: width,
            height: height,
            pos_x: pos_x,
            pos_y: pos_y,

            scroll: 0,
            cursor: Cursor::ElfHeader,
        }
    }

    pub fn mainloop(&mut self) -> GuiRet {
        loop {
            let ch = self.get_char();

            if ch == b'q' as i32 {
                return GuiRet::Break;
            } else if ch == b'\t' as i32 {
                return GuiRet::Switch;
            }

            self.draw();
        }
    }

    pub fn draw(&self) {
        // let top    = self.scroll;
        // let bottom = top + self.height;

        self.draw_elf_header();
    }

    pub fn keypressed(&mut self, ch : i32) {

    }

    fn draw_elf_header(&self) {
        nc::mvaddstr( self.pos_y, self.pos_x + 1, "Class:" );
    }

    fn get_char(&self) -> i32 {
        nc::getch()
    }
}

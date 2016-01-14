pub mod hex;
pub mod elf;

use self::elf::ElfGui;

use std::time::Duration;

use ncurses as nc;

pub struct Gui<'gui> {
    hex_gui : hex::HexGui<'gui>,
    elf_gui : Option<ElfGui>,
    gui_mod : Mod,

    width : i32,
    height : i32,
    pos_x : i32,
    pos_y : i32,
}

enum Mod { Hex, Elf }

pub enum GuiRet {
    Break, Switch
}

impl<'gui> Gui<'gui> {
    pub fn new_hex_gui(contents : &'gui Vec<u8>, path : &'gui str,
                       width : i32, height : i32, pos_x : i32, pos_y : i32) -> Gui<'gui> {
        Gui {
            hex_gui: hex::HexGui::new(contents, path, width, height, pos_x, pos_y),
            elf_gui: None,
            gui_mod: Mod::Hex,

            width: width, height: height, pos_x: pos_x, pos_y: pos_y,
        }
    }

    pub fn init_elf_gui(&mut self,
                        elf_header : ::parser::elf::ELFHeader,
                        program_headers : Vec<::parser::elf::ProgramHeader>,
                        section_headers : Vec<::parser::elf::SectionHeader>) {
        self.elf_gui = Some(ElfGui::new(elf_header, section_headers, program_headers,
                                        self.width, self.height, self.pos_x, self.pos_y));
    }

    pub fn mainloop(&mut self) {
        self.hex_gui.init();

        loop {
            match self.gui_mod {
                Mod::Hex => {
                    match self.hex_gui.mainloop() {
                        GuiRet::Break => { break; },
                        GuiRet::Switch => {
                            if self.elf_gui.is_some() {
                                self.gui_mod = Mod::Elf;
                                nc::clear();
                            } else {
                                self.hex_gui.notify(b"Not an ELF file!", Duration::new(2, 0));
                            }
                        },
                    }
                },
                Mod::Elf => {
                    let mut elf_gui = self.elf_gui.as_mut().unwrap();
                    match elf_gui.mainloop() {
                        GuiRet::Break => { break; },
                        GuiRet::Switch => {
                            self.gui_mod = Mod::Hex;
                            nc::clear();
                        },
                    }
                }
            }
        }
    }
}

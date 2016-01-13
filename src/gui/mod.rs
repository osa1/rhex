pub mod hex;
pub mod elf;

pub struct Gui<'gui> {
    hex_gui : hex::HexGui<'gui>,
    elf_gui : Option<elf::ElfGui<'gui>>,
    gui_mod : Mod,
}

enum Mod { Hex, Elf }

pub enum GuiRet {
    Break, Switch
}

impl<'gui> Gui<'gui> {
    pub fn new_hex_gui(contents : &'gui Vec<u8>, path : &'gui str) -> Gui<'gui> {
        Gui {
            hex_gui: hex::HexGui::new(contents, path),
            elf_gui: None,
            gui_mod: Mod::Hex,
        }
    }

    pub fn new_elf_gui(contents : &'gui Vec<u8>, path : &'gui str,
                       elf_header : ::parser::elf::ELFHeader,
                       program_headers : &'gui Vec<::parser::elf::ProgramHeader>,
                       section_headers : &'gui Vec<::parser::elf::SectionHeader>) -> Gui<'gui> {
        panic!()
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
                            }
                        },
                    }
                },
                Mod::Elf => {
                    let mut elf_gui = self.elf_gui.as_mut().unwrap();
                    match elf_gui.mainloop() {
                        GuiRet::Break => { break; },
                        GuiRet::Switch => { self.gui_mod = Mod::Hex; },
                    }
                }
            }
        }
    }
}

extern crate libc;
extern crate capstone;

use self::capstone::{Capstone, Instructions};
use self::capstone::capstone::CsResult;
use self::capstone::constants::{CsArch, CsMode};

use std::borrow::Borrow;
use std::cmp::min;
use std::io;
use std::io::Write;
use std::result::Result;

// TODO: Move this to gui
use gui::elf::widget::{Widget, WidgetRet};
use utils::draw_box;

use ncurses as nc;

pub struct DisasView {
    instrs : Vec<Instr>,
    // width : i32,
    // height : i32,
}

struct Instr {
    mnem : String,
    op_str : String,
    size : u16,
    addr : u64,
}

impl DisasView {
    pub fn new(code : &[u8] /* , width : i32, height : i32 */) -> DisasView {
        // Can't unwrap() because the error type (CsErr) is not an instance of
        // Debug. See https://github.com/richo/capstone-rs/issues/5
        match Capstone::new(CsArch::ARCH_X86, CsMode::MODE_64) {
            Err(err) => panic!("Can't instantiate Capstone: {}", err),
            Ok(capstone) => {
                let instrs = {
                    let ip = 0; // instruction pointer
                    let count = 0; // disassemble all
                    match capstone.disasm(code, ip, count) {
                        Err(err) => panic!("Can't disassemble: {}", err),
                        Ok(instrs) => instrs
                    }
                };

                DisasView {
                    instrs :
                        instrs.iter().map(|i| {

                            // writeln!(&mut io::stderr(), "{:?} - {:?}", i.mnemonic(), i.op_str());

                            Instr {
                                mnem : i.mnemonic().map_or("???".to_owned(), |s| s.to_string()),
                                op_str : i.op_str().map_or("???".to_owned(), |s| s.to_string()),
                                size : i.size,
                                addr : i.address,
                            }
                        }).collect(),
                    // width : width,
                    // height : height,
                }
            }
        }
    }
}

impl Widget for DisasView {

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, highlight : bool) {
        draw_box(pos_x, pos_y, width, height,
                 Some(format!("Disassembly: {}", self.instrs.len()).borrow()));
        for i in 0 .. min(height - 2, self.instrs.len() as i32) {
            nc::mvaddstr(pos_y + 1 + i, pos_x + 1, self.instrs[i as usize].mnem.borrow());
        }
    }

    fn get_height(&self) -> i32 {
        10 // FIXME: random
    }

    fn focus(&mut self) -> bool {
        false
    }

    fn keypressed(&mut self, key : i32) -> WidgetRet {
        WidgetRet::KeyIgnored
    }
}

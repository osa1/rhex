use std::borrow::Borrow;

use colors::Color;
use gui::elf::field;
use gui::elf::widget::{Widget, WidgetRet};
use parser::elf;
use utils::draw_box;

use ncurses as nc;

////////////////////////////////////////////////////////////////////////////////
// Program header type field

struct ProgramHeaderField {
    value : elf::ProgramHeaderType,
}

impl Widget for ProgramHeaderField {
    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let ty_str = "Header type:";

        nc::mvaddstr(pos_y, pos_x, ty_str);

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            let val_str = match self.value {
                elf::ProgramHeaderType::OS(u) => {
                    format!("Unknown (OS): 0x{:x}", u)
                },
                elf::ProgramHeaderType::PROC(u) => {
                    format!("Unknown (PROC): 0x{:x}", u)
                },
                other => {
                    format!("{:?}", other)
                }
            };

            nc::mvaddstr(pos_y, pos_x + ty_str.len() as i32 + 2, val_str.borrow());
        });
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct ProgramHeader {
    fields : Vec<Box<Widget>>,
    cursor : usize,
    has_focus : bool,
}

static HEADER_TITLE : &'static str = "Program header";

impl Widget for ProgramHeader {
    fn get_height(&self) -> i32 {
        10
    }

    fn focus(&mut self) -> bool {
        self.has_focus = true;
        true
    }

    fn keypressed(&mut self, key : i32) -> WidgetRet {
        if self.has_focus {
            self.keypressed_focus(key)
        } else {
            self.keypressed_no_focus(key)
        }
    }

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, highlight : bool) {
        if self.has_focus {
            nc::attron(Color::FrameActive.attr());
        } else if highlight {
            nc::attron(Color::FrameFocus.attr());
        }

        draw_box(pos_x, pos_y, width, height, Some(HEADER_TITLE));

        if self.has_focus {
            nc::attroff(Color::FrameActive.attr());
        } else if highlight {
            nc::attroff(Color::FrameFocus.attr());
        }

        for (field_idx, field) in self.fields.iter().enumerate() {
            let field_focus = field_idx == self.cursor && self.has_focus;
            field.draw(pos_x + 1, pos_y + 1 + field_idx as i32, width - 2, height - 2, field_focus);
        }
    }
}

impl ProgramHeader {
    fn keypressed_focus(&mut self, key : i32) -> WidgetRet {
        if key == 27 {
            self.has_focus = false;
            WidgetRet::LostFocus
        }

        else if key == nc::KEY_UP || key == b'k' as i32 {
            if self.cursor > 0 {
                self.cursor -= 1;
            }
            WidgetRet::KeyHandled
        }

        else if key == nc::KEY_DOWN || key == b'j' as i32 {
            if self.cursor < self.fields.len() - 1 {
                self.cursor += 1;
            }
            WidgetRet::KeyHandled
        }

        else {
            WidgetRet::KeyHandled
        }
    }

    fn keypressed_no_focus(&mut self, key : i32) -> WidgetRet {
        WidgetRet::KeyIgnored
    }
}

pub fn mk_pgm_hdr_fields(hdrs : &Vec<elf::ProgramHeader>) -> Vec<Box<Widget>> {
    let mut headers : Vec<Box<Widget>> = Vec::with_capacity(hdrs.len());

    for hdr in hdrs {
        let mut fields : Vec<Box<Widget>> = Vec::with_capacity(9);

        fields.push(Box::new(ProgramHeaderField {
            value: hdr.ty,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.offset,
            title: "Offset:".to_string(),
            num_fields: 8,
            current_field: 0,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.vaddr,
            title: "Virtual address:".to_string(),
            num_fields: 8,
            current_field: 1,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.paddr,
            title: "Physical address".to_string(),
            num_fields: 8,
            current_field: 2,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.filesz,
            title: "File size:".to_string(),
            num_fields: 8,
            current_field: 3,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.memsz,
            title: "Memory size".to_string(),
            num_fields: 8,
            current_field: 4,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u32> {
            value: hdr.flags,
            title: "Flags:".to_string(),
            num_fields: 8,
            current_field: 5,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.align,
            title: "Align:".to_string(),
            num_fields: 8,
            current_field: 6,
        }));

        headers.push(Box::new(ProgramHeader {
            fields: fields,
            cursor: 0,
            has_focus: false,
        }));
    }

    headers
}

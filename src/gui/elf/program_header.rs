use colors::Color;
use gui::elf::field;
use parser::elf;
use utils::draw_box;

use ncurses as nc;

pub struct ProgramHeader {
    fields : Vec<Box<field::Field>>,
    cursor : usize,
    has_focus : bool,
}

pub enum ProgramHeaderRet {
    LostFocus, KeyHandled, KeyIgnored,
}

static HEADER_TITLE : &'static str = "Program header";

impl ProgramHeader {
    pub fn get_height(&self) -> i32 {
        8
    }

    pub fn keypressed(&mut self, ch : i32) -> ProgramHeaderRet {
        if self.has_focus {
            self.keypressed_focus(ch)
        } else {
            self.keypressed_no_focus(ch)
        }
    }

    fn keypressed_focus(&mut self, ch : i32) -> ProgramHeaderRet {
        if ch == 27 {
            self.has_focus = false;
            ProgramHeaderRet::LostFocus
        } else {
            ProgramHeaderRet::KeyHandled
        }
    }

    fn keypressed_no_focus(&mut self, ch : i32) -> ProgramHeaderRet {
        ProgramHeaderRet::KeyIgnored
    }

    pub fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, highlight : bool) {
        // FIXME: Figure out how to export/import macros and use with_attr!
        // here.

        if highlight {
            nc::attron(Color::FrameFocus.attr());
        }

        draw_box(pos_x, pos_y, width, height, Some(HEADER_TITLE));

        if highlight {
            nc::attroff(Color::FrameFocus.attr());
        }

        for (field_idx, field) in self.fields.iter().enumerate() {
            field.draw(pos_x + 1, pos_y + 1 + field_idx as i32, width - 2, height - 2, false);
        }
    }
}

pub fn mk_pgm_hdr_fields(hdrs : &Vec<elf::ProgramHeader>) -> Vec<ProgramHeader> {
    let mut headers = Vec::with_capacity(hdrs.len());

    for hdr in hdrs {
        let mut fields : Vec<Box<field::Field>> = Vec::with_capacity(8);

        // TODO: need a field for ProgramHeaderType

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

        headers.push(ProgramHeader {
            fields: fields,
            cursor: 0,
            has_focus: false,
        });
    }

    headers
}

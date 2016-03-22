use std::borrow::Borrow;
use std::str;

use colors::Color;
use parser::elf;
use utils::draw_box;

use gui::disas::DisasView;
use gui::elf::field;
use gui::elf::widget::{Widget, WidgetRet};

use ncurses as nc;

////////////////////////////////////////////////////////////////////////////////

pub struct SectionHeader {
    fields : Vec<Box<Widget>>,
    cursor : usize,
    has_focus : bool,
}

static HEADER_TITLE : &'static str = "Section header";

impl Widget for SectionHeader {
    fn get_height(&self) -> i32 {
        self.fields.iter().map(|f| f.get_height()).sum::<i32>() + 2
    }

    fn focus(&mut self) -> bool {
        // self.has_focus = true;
        false
    }

    fn keypressed(&mut self, key : i32) -> WidgetRet {
        WidgetRet::KeyIgnored
    }

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, highlight : bool) {
        let attr = if self.has_focus {
            Color::FrameActive.attr()
        } else if highlight {
            Color::FrameFocus.attr()
        } else {
            0
        };

        with_attr!(true, attr, {
            draw_box(pos_x, pos_y, width, height, Some(HEADER_TITLE));
        });

        for (field_idx, field) in self.fields.iter().enumerate() {
            let field_focus = field_idx == self.cursor && self.has_focus;
            field.draw(pos_x + 1, pos_y + 1 + field_idx as i32, width - 2, height - 2, field_focus);
        }
    }
}

pub fn mk_sec_hdr_fields(hdrs : &Vec<elf::SectionHeader>, string_table : &Option<elf::StringTable>)
                         -> Vec<Box<Widget>> {
    let mut headers : Vec<Box<Widget>> = Vec::with_capacity(hdrs.len());

    for hdr in hdrs {
        let mut fields : Vec<Box<Widget>> = Vec::with_capacity(9);

        fields.push(Box::new(field::ElfHdrField_str {
            value: format!("{:?}", hdr.ty),
            title: "Type:".to_string(),
            num_fields: 10,
            current_field: 1,
        }));

        let name_string =
            if hdr.name == 0 {
                None
            } else {
                Some(
                    if let Some(ref tbl) = *string_table {
                        if let Some(bytes) = elf::index_string_table(tbl, hdr.name as usize) {
                            if let Ok(str) = str::from_utf8(bytes) {
                                str.to_string()
                            } else {
                                "<Non-utf8 string>".to_string()
                            }
                        } else {
                            "<Can't read from string table>".to_string()
                        }
                    } else {
                        "<String table missing>".to_string()
                    })
            };

        for s in name_string.iter() {
            fields.push(Box::new(field::ElfHdrField_str {
                value: s.clone(),
                title: "Name:".to_string(),
                num_fields: 10,
                current_field: 2,
            }));
        }

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.flags,
            title: "Flags:".to_string(),
            num_fields: 10,
            current_field: 3,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.addr,
            title: "Addr:".to_string(),
            num_fields: 10,
            current_field: 4,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.offset,
            title: "Offset:".to_string(),
            num_fields: 10,
            current_field: 5,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.size,
            title: "Size:".to_string(),
            num_fields: 10,
            current_field: 6,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u32> {
            value: hdr.link,
            title: "Link:".to_string(),
            num_fields: 10,
            current_field: 7,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u32> {
            value: hdr.info,
            title: "Info:".to_string(),
            num_fields: 10,
            current_field: 8,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.addralign,
            title: "Addralign:".to_string(),
            num_fields: 10,
            current_field: 9,
        }));

        fields.push(Box::new(field::ElfHdrField_hex::<u64> {
            value: hdr.entsize,
            title: "Entsize:".to_string(),
            num_fields: 10,
            current_field: 10,
        }));

        if name_string == Some(".text".to_owned()) {
            fields.push(Box::new(DisasView::new(hdr.contents)));
        }

        headers.push(Box::new(SectionHeader {
            fields: fields,
            cursor: 0,
            has_focus: false,
        }));
    }

    headers
}

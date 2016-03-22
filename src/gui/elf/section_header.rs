use std::borrow::Borrow;

use colors::Color;
use gui::elf::field;
use gui::elf::widget::{Widget, WidgetRet};
use parser::elf;
use utils::draw_box;

use ncurses as nc;

////////////////////////////////////////////////////////////////////////////////

pub struct SectionHeader {
    fields : Vec<Box<Widget>>,
    cusror : usize,
    has_focus : bool,
}

static HEADER_TITLE : &'static str = "Section header";

impl Widget for SectionHeader {
    fn get_height(&self) -> i32 {
        9
    }

    fn focus(&mut self) -> bool {
        self.has_focus = true;
        true
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
    }
}

pub fn mk_sec_hdr_fields(hdrs : &Vec<elf::SectionHeader>) -> Vec<Box<Widget>> {
    let mut headers : Vec<Box<Widget>> = Vec::with_capacity(hdrs.len());

    // for hdr in hdrs {
    //     let mut fields : Vec<Box<Widget>> = Vec::with_capacity(9);

    //     fields.push(Box::new(field::ElfHdrField_str {
    //         value: format!("{:?}", hdr.ty),
    //         title: "Type:".to_string(),
    //         num_fields: 9,
    //         current_field: 1,
    //     }));
    // }

    headers
}

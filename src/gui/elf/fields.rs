use parser::elf;

use colors::Color;

use ncurses as nc;

pub enum FieldRet {
    Prev, Field(usize), Next
}

pub trait Field {
    /// Get cursor's value for this field.
    fn get_idx(&self) -> usize;

    /// Render the field.
    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool);

    /// Get cursor value for the next field.
    fn next(&self) -> FieldRet;

    /// Get cursor value for the previous field.
    fn prev(&self) -> FieldRet;
}

////////////////////////////////////////////////////////////////////////////////
// Class

struct ElfHdrField_Class {
    value : elf::Class,
}

impl Field for ElfHdrField_Class {
    fn get_idx(&self) -> usize { 0 }

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let class_str = "Class:";

        nc::mvaddstr(pos_y, pos_x, class_str);

        let class_val_str = match self.value {
            elf::Class::Bit32 => "32 bit",
            elf::Class::Bit64 => "64 bit",
        };

        if focus {
            nc::attron( nc::A_BOLD() | Color::CursorFocus.attr() );
        }

        nc::mvaddstr(pos_y, pos_x + class_str.len() as i32 + 2, class_val_str);

        if focus {
            nc::attroff( nc::A_BOLD() | Color::CursorFocus.attr() );
        }
    }

    fn next(&self) -> FieldRet {
        FieldRet::Field(1)
    }

    fn prev(&self) -> FieldRet {
        FieldRet::Prev
    }
}

////////////////////////////////////////////////////////////////////////////////
// Endianness

struct ElfHdrField_Endianness {
    value : elf::Endianness,
}

impl Field for ElfHdrField_Endianness {
    fn get_idx(&self) -> usize { 1 }

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let endianness_str = "Endianness:";

        nc::mvaddstr(pos_y, pos_x, endianness_str);

        let endianness_val_str = match self.value {
            elf::Endianness::LittleEndian => "Little endian",
            elf::Endianness::BigEndian => "Big endian",
        };

        if focus {
            nc::attron( nc::A_BOLD() | Color::CursorFocus.attr() );
        }

        nc::mvaddstr(pos_y, pos_x + endianness_str.len() as i32 + 2, endianness_val_str);

        if focus {
            nc::attroff( nc::A_BOLD() | Color::CursorFocus.attr() );
        }
    }

    fn next(&self) -> FieldRet {
        FieldRet::Field(2)
    }

    fn prev(&self) -> FieldRet {
        FieldRet::Field(0)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Generate field vector

pub fn mk_elf_hdr_fields(hdr : &elf::ELFHeader) -> Vec<Box<Field>> {
    vec![
        Box::new(ElfHdrField_Class { value: hdr.class }),
        Box::new(ElfHdrField_Endianness { value: hdr.endianness }),
    ]
}

use std::borrow::Borrow;

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

macro_rules! with_attr {
    ( $guard:expr, $attr_expr:expr, $body:expr ) => {
        if $guard {
            nc::attron($attr_expr);
        }

        $body;

        if $guard {
            nc::attroff($attr_expr);
        }
    };
}

// It turns out Rust macros kinda suck. I can't evaluate expression in macro
// expansion time, I can't match only integer literals (the book lists 10 things
// that we can match, integer literals are not one of them) etc. so if I want to
// define get_idx(), next() and prev() here using a macro, I'd have to add some
// runtime overhead, by doing something like this:

macro_rules! mk_boring_fns {
    ( $last_field:expr, $current_field:expr ) => {

        fn get_idx(&self) -> usize { $current_field }

        fn next(&self) -> FieldRet {
            if $current_field == $last_field {
                FieldRet::Next
            } else {
                FieldRet::Field($current_field + 1)
            }
        }

        fn prev(&self) -> FieldRet {
            if $current_field == 0 {
                FieldRet::Prev
            } else {
                FieldRet::Field($current_field - 1)
            }
        }

    };
}

// The problem here is that prev() and next() methods have runtime comparisons
// for no reason.
//
// However, I think the code generator should be smart enough to do some
// evaluation here and reduce these expressions. So.. I ended up using this.
//
// TODO: Check the optimized LLVM IR. (I don't think there's a way to do this
// using Cargo at the moment)

////////////////////////////////////////////////////////////////////////////////
// Class

struct ElfHdrField_Class {
    value : elf::Class,
}

impl Field for ElfHdrField_Class {
    mk_boring_fns!(14, 0);

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let class_str = "Class:";

        nc::mvaddstr(pos_y, pos_x, class_str);

        let class_val_str = match self.value {
            elf::Class::Bit32 => "32 bit",
            elf::Class::Bit64 => "64 bit",
        };

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            nc::mvaddstr(pos_y, pos_x + class_str.len() as i32 + 2, class_val_str);
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
// Endianness

struct ElfHdrField_Endianness {
    value : elf::Endianness,
}

impl Field for ElfHdrField_Endianness {
    mk_boring_fns!(14, 1);

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let endianness_str = "Endianness:";

        nc::mvaddstr(pos_y, pos_x, endianness_str);

        let endianness_val_str = match self.value {
            elf::Endianness::LittleEndian => "Little endian",
            elf::Endianness::BigEndian => "Big endian",
        };

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            nc::mvaddstr(pos_y, pos_x + endianness_str.len() as i32 + 2, endianness_val_str);
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
// Generate field vector

struct ElfHdrField_ABI {
    value : elf::OsABI,
}

impl Field for ElfHdrField_ABI {
    mk_boring_fns!(14, 2);

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let abi_str = "ABI:";

        nc::mvaddstr(pos_y, pos_x, abi_str);

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            let endianness_val_str = format!("{:?}", self.value);
            nc::mvaddstr(pos_y, pos_x + abi_str.len() as i32 + 2, endianness_val_str.borrow());
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
// Generate field vector

pub fn mk_elf_hdr_fields(hdr : &elf::ELFHeader) -> Vec<Box<Field>> {
    vec![
        Box::new(ElfHdrField_Class { value: hdr.class }),
        Box::new(ElfHdrField_Endianness { value: hdr.endianness }),
        Box::new(ElfHdrField_ABI { value: hdr.abi }),
    ]
}

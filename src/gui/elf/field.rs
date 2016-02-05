// TODO: This has too much repetition. We should at least define generic
// renderers for field types, e.g. all u64 fields should use same renderer.

use std::borrow::Borrow;
use std::fmt::LowerHex;

use colors::Color;
use parser::elf;

use ncurses as nc;

pub trait Field {
    /// Render the field.
    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool);
}

////////////////////////////////////////////////////////////////////////////////
// Some generic field structs for repeatedly-used field types

pub struct ElfHdrField_hex<T : LowerHex> {
    pub value : T,
    pub title : String,

    pub num_fields : usize,
    pub current_field : usize,
}

impl<T : LowerHex> Field for ElfHdrField_hex<T> {
    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        nc::mvaddstr(pos_y, pos_x, self.title.borrow());

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            let val_str = format!("0x{:x}", self.value);
            nc::mvaddstr(pos_y, pos_x + self.title.len() as i32 + 2, val_str.borrow());
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
// Class

struct ElfHdrField_Class {
    value : elf::Class,
}

impl Field for ElfHdrField_Class {
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
// ABI

struct ElfHdrField_ABI {
    value : elf::OsABI,
}

impl Field for ElfHdrField_ABI {
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
// Object type

struct ElfHdrField_ObjType {
    value : elf::ObjType,
}

impl Field for ElfHdrField_ObjType {
    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let obj_type_str = "Object type:";

        nc::mvaddstr(pos_y, pos_x, obj_type_str);

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            let obj_type_val_str = format!("{:?}", self.value);
            nc::mvaddstr(pos_y, pos_x + obj_type_str.len() as i32 + 2, obj_type_val_str.borrow());
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
// ISA

struct ElfHdrField_ISA {
    value : elf::ISA,
}

impl Field for ElfHdrField_ISA {
    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, focus : bool) {
        let isa_str = "ISA:";

        nc::mvaddstr(pos_y, pos_x, isa_str);

        with_attr!(focus, nc::A_BOLD() | Color::CursorFocus.attr(), {
            let isa_val_str = format!("{:?}", self.value);
            nc::mvaddstr(pos_y, pos_x + isa_str.len() as i32 + 2, isa_val_str.borrow());
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
        Box::new(ElfHdrField_ObjType { value: hdr.obj_type }),
        Box::new(ElfHdrField_ISA { value: hdr.isa }),
        Box::new(ElfHdrField_hex::<u64> {
            value: hdr.entry_addr,
            title: "Entry address:".to_string(),
            num_fields: 15,
            current_field: 5,
        }),
        Box::new(ElfHdrField_hex::<u64> {
            value: hdr.phoff,
            title: "Program header offset:".to_string(),
            num_fields: 15,
            current_field: 6,
        }),
        Box::new(ElfHdrField_hex::<u64> {
            value: hdr.shoff,
            title: "Section header offset:".to_string(),
            num_fields: 15,
            current_field: 7,
        }),
        Box::new(ElfHdrField_hex::<u32> {
            value: hdr.flags,
            title: "Flags:".to_string(),
            num_fields: 15,
            current_field: 8,
        }),
        Box::new(ElfHdrField_hex::<u16> {
            value: hdr.ehsize,
            title: "ELF header size:".to_string(),
            num_fields: 15,
            current_field: 9,
        }),
        Box::new(ElfHdrField_hex::<u16> {
            value: hdr.phentsize,
            title: "Program header entry size:".to_string(),
            num_fields: 15,
            current_field: 10,
        }),
        Box::new(ElfHdrField_hex::<u16> {
            value: hdr.phnum,
            title: "# of program headers:".to_string(),
            num_fields: 15,
            current_field: 11,
        }),
        Box::new(ElfHdrField_hex::<u16> {
            value: hdr.shentsize,
            title: "Section header entry size:".to_string(),
            num_fields: 15,
            current_field: 12,
        }),
        Box::new(ElfHdrField_hex::<u16> {
            value: hdr.shnum,
            title: "# of section headers".to_string(),
            num_fields: 15,
            current_field: 13,
        }),
        Box::new(ElfHdrField_hex::<u16> {
            value: hdr.shnum,
            title: "Section name string table idx:".to_string(),
            num_fields: 15,
            current_field: 14,
        }),
    ]
}

use parser::elf;
use gui::elf::fields;

use ncurses as nc;

pub struct ProgramHeader {
    pub fields: Vec<Box<fields::Field>>,
}

impl ProgramHeader {
    pub fn get_height(&self) -> i32 {
        8
    }
}

pub fn mk_pgm_hdr_fields(hdrs : &Vec<elf::ProgramHeader>) -> Vec<ProgramHeader> {
    let mut headers = Vec::with_capacity(hdrs.len());

    for hdr in hdrs {
        let mut fields : Vec<Box<fields::Field>> = Vec::with_capacity(8);

        // TODO: need a field for ProgramHeaderType

        fields.push(Box::new(fields::ElfHdrField_hex::<u64> {
            value: hdr.offset,
            title: "Offset:".to_string(),
            num_fields: 8,
            current_field: 0,
        }));

        fields.push(Box::new(fields::ElfHdrField_hex::<u64> {
            value: hdr.vaddr,
            title: "Virtual address:".to_string(),
            num_fields: 8,
            current_field: 1,
        }));

        fields.push(Box::new(fields::ElfHdrField_hex::<u64> {
            value: hdr.paddr,
            title: "Physical address".to_string(),
            num_fields: 8,
            current_field: 2,
        }));

        fields.push(Box::new(fields::ElfHdrField_hex::<u64> {
            value: hdr.filesz,
            title: "File size:".to_string(),
            num_fields: 8,
            current_field: 3,
        }));

        fields.push(Box::new(fields::ElfHdrField_hex::<u64> {
            value: hdr.memsz,
            title: "Memory size".to_string(),
            num_fields: 8,
            current_field: 4,
        }));

        fields.push(Box::new(fields::ElfHdrField_hex::<u32> {
            value: hdr.flags,
            title: "Flags:".to_string(),
            num_fields: 8,
            current_field: 5,
        }));

        fields.push(Box::new(fields::ElfHdrField_hex::<u64> {
            value: hdr.align,
            title: "Align:".to_string(),
            num_fields: 8,
            current_field: 6,
        }));

        headers.push(ProgramHeader {
            fields: fields,
        });
    }

    headers
}

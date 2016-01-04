use std::borrow::Borrow;
use std::clone::Clone;
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub enum ParseResult {
    ParseOK(ELFHeader),
    NotELF,
    CantReadFile(Error),
    CantParse,
}

#[derive(Debug)]
pub enum Class { Bit32, Bit64 }

#[derive(Debug, Clone)]
pub enum Endianness { LittleEndian, BigEndian }

#[derive(Debug)]
pub enum OsABI { SystemV, HPUX, NetBSD, Linux, Solaris, AIX, IRIX, FreeBSD, OpenBSD, OpenVMS }

#[derive(Debug)]
pub enum ObjType { Relocatable, Executable, Shared, Core }

#[derive(Debug)]
pub enum ISA { NA, SPARC, X86, MIPS, PowerPC, ARM, SuperH, IA64, X86_64, AArch64 }

/// ELF header.
#[derive(Debug)]
pub struct ELFHeader {
    class: Class,
    endianness: Endianness,
    abi: OsABI,
    obj_type: ObjType,
    isa: ISA,

    /// Virtual address to which the system first transfers control, thus
    /// starting the process. Zero if the file has no associated entry point.
    entry_addr: u64,

    /// The program header table's file offset in bytes. Zero if the file has no
    /// program header table.
    phoff: u64,

    /// The section header table's file offset in bytes. Zero if the file has no
    /// section header table.
    shoff: u64,

    /// Processor-specific flags associated with the file.
    flags: u32,

    /// The ELF header's size in bytes.
    /// (Normally 64 bytes for 64-bit and 52 for 32-bit format)
    ehsize: u16,

    /// The size in bytes of one entry in the file's program header table; all
    /// entries are the same size.
    phentsize: u16,

    /// The number of entries in the program header table. Thus the product of
    /// `phentsize` and `phnum` gives the program header table's size in bytes.
    /// Zero if the file has no program header table.
    phnum: u16,

    /// The size in bytes of one entry in the file's section header table; all
    /// entries are the same size.
    shentsize: u16,

    /// The number of entries in the section header table. Thus the product of
    /// `shentsize` and `shnum` gives the section header table's size in bytes.
    /// Zero if the file has no section header table.
    shnum: u16,

    /// The section header table index of the entry associated with the section
    /// name string table. Zero if the file has no section name string table.
    shstrndx: u16,
}

pub fn parse(path : &Path) -> ParseResult {
    let mut contents = Vec::new();

    match File::open(path) {
        Err(err) => ParseResult::CantReadFile(err),
        Ok(mut file) => {
            file.read_to_end(&mut contents);
            parse_contents(contents.borrow())
        }
    }
}

fn parse_contents(contents : &[u8]) -> ParseResult {
    let mag0 = contents[0];
    let mag1 = contents[1];
    let mag2 = contents[2];
    let mag3 = contents[3];

    if !(mag0 == 0x7F && mag1 == b'E' && mag2 == b'L' && mag3 == b'F') {
        return ParseResult::NotELF;
    }

    let class =
        match contents[4] {
            1 => Class::Bit32,
            2 => Class::Bit64,
            _ => return ParseResult::CantParse,
        };

    let endianness =
        match contents[5] {
            1 => Endianness::LittleEndian,
            2 => Endianness::BigEndian,
            _ => return ParseResult::CantParse,
        };

    // Skipping offset 6

    let os_abi =
        match contents[7] {
            0x00 => OsABI::SystemV, 0x01 => OsABI::HPUX, 0x02 => OsABI::NetBSD, 0x03 => OsABI::Linux,
            0x06 => OsABI::Solaris, 0x07 => OsABI::AIX, 0x08 => OsABI::IRIX, 0x09 => OsABI::FreeBSD,
            0x0C => OsABI::OpenBSD, 0x0D => OsABI::OpenVMS,
            _ => return ParseResult::CantParse,
        };

    // Skipping offset 8, 9

    let obj_type =
        match read_u16(endianness.clone(), &contents[ 0x10 .. ]) {
            1 => ObjType::Relocatable,
            2 => ObjType::Executable,
            3 => ObjType::Shared,
            4 => ObjType::Core,
            _ => return ParseResult::CantParse,
        };

    let isa =
        match read_u16(endianness.clone(), &contents[ 0x12 .. ]) {
            0x00 => ISA::NA,
            0x02 => ISA::SPARC,
            0x03 => ISA::X86,
            0x08 => ISA::MIPS,
            0x14 => ISA::PowerPC,
            0x28 => ISA::ARM,
            0x2A => ISA::SuperH,
            0x32 => ISA::IA64,
            0x3E => ISA::X86_64,
            0xB7 => ISA::AArch64,
            _ => return ParseResult::CantParse,
        };

    // Skipping offset 0x14

    let entry_addr =
        match class {
            Class::Bit32 => {
                read_u32(endianness.clone(), &contents[ 0x18 .. ]) as u64
            },
            Class::Bit64 => {
                read_u64(endianness.clone(), &contents[ 0x18 .. ])
            }
        };

    let phoff =
        match class {
            Class::Bit32 => {
                read_u32(endianness.clone(), &contents[ 0x1C .. ]) as u64
            },
            Class::Bit64 => {
                read_u64(endianness.clone(), &contents[ 0x20 .. ])
            }
        };

    let shoff =
        match class {
            Class::Bit32 => {
                read_u32(endianness.clone(), &contents[ 0x20 .. ]) as u64
            },
            Class::Bit64 => {
                read_u64(endianness.clone(), &contents[ 0x28 .. ])
            }
        };

    let flags =
        match class {
            Class::Bit32 => {
                read_u32(endianness.clone(), &contents[ 0x24 .. ])
            },
            Class::Bit64 => {
                read_u32(endianness.clone(), &contents[ 0x30 .. ])
            }
        };

    let header_size =
        match class {
            Class::Bit32 => {
                read_u32(endianness.clone(), &contents[ 0x24 .. ])
            },
            Class::Bit64 => {
                read_u32(endianness.clone(), &contents[ 0x30 .. ])
            }
        };

    let ehsize =
        match class {
            Class::Bit32 => {
                read_u16(endianness.clone(), &contents[ 0x28 .. ])
            },
            Class::Bit64 => {
                read_u16(endianness.clone(), &contents[ 0x34 .. ])
            }
        };

    let phentsize =
        match class {
            Class::Bit32 => {
                read_u16(endianness.clone(), &contents[ 0x2A .. ])
            },
            Class::Bit64 => {
                read_u16(endianness.clone(), &contents[ 0x36 .. ])
            }
        };

    let phnum =
        match class {
            Class::Bit32 => {
                read_u16(endianness.clone(), &contents[ 0x2C .. ])
            },
            Class::Bit64 => {
                read_u16(endianness.clone(), &contents[ 0x38 .. ])
            }
        };

    let shentsize =
        match class {
            Class::Bit32 => {
                read_u16(endianness.clone(), &contents[ 0x2E .. ])
            },
            Class::Bit64 => {
                read_u16(endianness.clone(), &contents[ 0x3A .. ])
            }
        };

    let shnum =
        match class {
            Class::Bit32 => {
                read_u16(endianness.clone(), &contents[ 0x30 .. ])
            },
            Class::Bit64 => {
                read_u16(endianness.clone(), &contents[ 0x3C .. ])
            }
        };

    let shstrndx =
        match class {
            Class::Bit32 => {
                read_u16(endianness.clone(), &contents[ 0x32 .. ])
            },
            Class::Bit64 => {
                read_u16(endianness.clone(), &contents[ 0x3E .. ])
            }
        };

    ParseResult::ParseOK(ELFHeader {
        class: class,
        endianness: endianness,
        abi: os_abi,
        obj_type: obj_type,
        isa: isa,
        entry_addr: entry_addr,
        phoff: phoff,
        shoff: shoff,
        flags: flags,
        ehsize: ehsize,
        phentsize: phentsize,
        phnum: phnum,
        shentsize: shentsize,
        shnum: shnum,
        shstrndx: shstrndx,
    })
}

fn read_u16(endianness : Endianness, from : &[u8]) -> u16 {
    match endianness {
        Endianness::LittleEndian => {
            ((from[1] as u16) << 8) | (from[0] as u16)
        },
        Endianness::BigEndian => {
            ((from[0] as u16) << 8) | (from[1] as u16)
        }
    }
}

fn read_u32(endianness : Endianness, from : &[u8]) -> u32 {
    match endianness {
        Endianness::LittleEndian => {
            ((read_u16(endianness.clone(), &from[ 2 ..  ]) as u32) << 16)
                | (read_u16(endianness.clone(), from) as u32)
        },
        Endianness::BigEndian => {
            ((read_u16(endianness.clone(), from) as u32) << 16)
                | (read_u16(endianness.clone(), &from[ 2 .. ]) as u32)
        }
    }
}

fn read_u64(endianness : Endianness, from : &[u8]) -> u64 {
    match endianness {
        Endianness::LittleEndian => {
            ((read_u32(endianness.clone(), &from[ 4 .. ]) as u64) << 32)
                | (read_u32(endianness.clone(), from) as u64)
        },
        Endianness::BigEndian => {
            ((read_u32(endianness.clone(), from) as u64) << 32)
                | (read_u32(endianness.clone(), &from[ 4 .. ]) as u64)
        }
    }
}

/// Specification and parsing of ELF files. Documentation mostly copied from the
/// man page.

use std::borrow::Borrow;
use std::clone::Clone;
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////
// Specification of ELF format
////////////////////////////////////////////////////////////////////////////////

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

#[derive(Debug)]
struct SectionHeader32 {
    /// Name of the section. Its value is an index into the section header
    /// string table section, giving the location of a null-terminated string.
    name: u32,

    /// Categorizes the section's contents and semantics.
    ty: SectionHeaderType,

    flags: u32,

    /// If the section will appear in the memory image of a process, this is the
    /// address at which the section's first byte should reside. Otherwise it's 0.
    addr: u32,

    /// The byte offset from the beginning of the file to the first byte in the
    /// section.
    offset: u32,

    /// The section's size in bytes.
    size: u32,

    /// Section header table index link.
    // ???
    link: u32,

    /// Extra information, whose interpretation depends on the section type.
    info: u32,

    /// Alignment constraints.
    addralign: u32,

    /// Some sections hold a table of fixed-size entries, such as a symbol
    /// table. For such a section, this gives the size in bytes of each entry.
    /// 0 if the section does not hold a table of fixed-size entries.
    entsize: u32,
}

/// See documentation of `SectionHeader64`.
#[derive(Debug)]
struct SectionHeader64 {
    name: u32,
    ty: SectionHeaderType,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entsize: u64,
}

#[derive(Debug)]
enum SectionHeaderType {
    /// This marks the section header as inactive. It does not have an
    /// associated section. Other members of the section header have undefined
    /// values.
    NULL,

    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    PROGBITS,

    /// The section holds a symbol table. Typically, `SYMTAB` provides symbols
    /// for link editing, though it may also be used for dynamic linking. As a
    /// complete symbol table, it may contain many symbols unnecessary for
    /// dynamic linking. An object file can also contain a `DYNSYM` section.
    SYMTAB,

    /// The section holds a string table. An object file may have multiple
    /// string table sections.
    STRTAB,

    /// The section holds relocation entries with explicit addends An object may
    /// have multiple relocation sections.
    RELA,

    /// The section holds a symbol hash table. An object participating in
    /// dynamic linking must contain a symbol hash table. An object file may
    /// have only one hash table.
    HASH,

    /// The section holds information for dynamic linking. An object file may
    /// have only one dynamic section.
    DYNAMIC,

    /// The section holds information that marks the file in some way.
    NOTE,

    /// A section of this type occupies no space in the file but otherwise
    /// resembles `PROGBITS`. Although this section contains no bytes, the
    /// `offset` member contains the conceptual file offset.
    NOBITS,

    /// The section holds relocation offsets without explicit addends. An
    /// object file may have multiple relocation sections.
    REL,

    /// The section is reserved but has unspecified semantics.
    SHLIB,

    /// The section holds a minimal set of dynamic linking symbols. An object
    /// file can also contain a `SYMTAB` section.
    DYNSYM,

    /// This value up to and including `HIPROC` is reserved for
    /// processor-specific semantics.
    LOPROC,

    /// This value down to and including `LOPROC` is reserved for
    /// processor-specific semantics.
    HIPROC,

    /// This value specifies the lower bound of the range of indices reserved
    /// for application programs.
    LOUSER,

    /// This value specifies the upper bound of the range of indices reserved
    /// for application programs. Section types between `LOUSER` and `HIUSER`
    /// may be used by the application, without conflicting with current or
    /// future system-defined section types.
    HIUSER,
}

////////////////////////////////////////////////////////////////////////////////
// Parsing
////////////////////////////////////////////////////////////////////////////////

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

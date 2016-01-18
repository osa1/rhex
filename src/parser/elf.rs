/// Specification and parsing of ELF files. Documentation mostly copied from the
/// man page.

use std::borrow::Borrow;
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////
// Specification of ELF format
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum Class { Bit32, Bit64 }

#[derive(Debug, Clone, Copy)]
pub enum Endianness { LittleEndian, BigEndian }

#[derive(Debug, Clone, Copy)]
pub enum OsABI { SystemV, HPUX, NetBSD, Linux, Solaris, AIX, IRIX, FreeBSD, OpenBSD, OpenVMS }

#[derive(Debug, Clone, Copy)]
pub enum ObjType { Relocatable, Executable, Shared, Core }

#[derive(Debug, Clone, Copy)]
pub enum ISA { NA, SPARC, X86, MIPS, PowerPC, ARM, SuperH, IA64, X86_64, AArch64 }

#[derive(Debug)]
pub struct ELFHeader {
    pub class: Class,
    pub endianness: Endianness,
    pub abi: OsABI,
    pub obj_type: ObjType,
    pub isa: ISA,

    /// Virtual address to which the system first transfers control, thus
    /// starting the process. Zero if the file has no associated entry point.
    pub entry_addr: u64,

    /// The program header table's file offset in bytes. Zero if the file has no
    /// program header table.
    pub phoff: u64,

    /// The section header table's file offset in bytes. Zero if the file has no
    /// section header table.
    pub shoff: u64,

    /// Processor-specific flags associated with the file.
    pub flags: u32,

    /// The ELF header's size in bytes.
    /// (Normally 64 bytes for 64-bit and 52 for 32-bit format)
    pub ehsize: u16,

    /// The size in bytes of one entry in the file's program header table; all
    /// entries are the same size.
    pub phentsize: u16,

    /// The number of entries in the program header table. Thus the product of
    /// `phentsize` and `phnum` gives the program header table's size in bytes.
    /// Zero if the file has no program header table.
    pub phnum: u16,

    /// The size in bytes of one entry in the file's section header table; all
    /// entries are the same size.
    pub shentsize: u16,

    /// The number of entries in the section header table. Thus the product of
    /// `shentsize` and `shnum` gives the section header table's size in bytes.
    /// Zero if the file has no section header table.
    pub shnum: u16,

    /// The section header table index of the entry associated with the section
    /// name string table. Zero if the file has no section name string table.
    pub shstrndx: u16,
}

#[derive(Debug)]
pub enum ProgramHeader { ProgramHeader32(ProgramHeader32), ProgramHeader64(ProgramHeader64) }

#[derive(Debug)]
pub struct ProgramHeader32 {
    /// What kind of segment this array element describes or how to interpret
    /// the array element's information.
    pub ty: ProgramHeaderType,

    /// Holds the offset from the beginning of the file at which the first byte
    /// of the segment resides.
    pub offset: u32,

    /// Holds the virtual address at which the first byte of the segment resides
    /// in memory.
    pub vaddr: u32,

    /// On systems for which physical addressing is relevant, this member is
    /// reserved for the segment's physical address. Under BSD this member is
    /// not used and must be zero.
    pub paddr: u32,

    /// Holds the number of bytes in the file image of the segment. It may be
    /// zero.
    pub filesz: u32,

    /// Holds the number of bytes in the memory image of the segment. It may be
    /// zero.
    pub memsz: u32,

    /// TODO
    pub flags: u32,

    /// TODO
    pub align: u32,
}

/// See documentation of `ProgramHeader32`.
#[derive(Debug)]
pub struct ProgramHeader64 {
    pub ty: ProgramHeaderType,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

#[derive(Debug, Clone)]
pub enum ProgramHeaderType {
    /// The array element is unused and the other members' values are undefined.
    /// This lets the program header have ignored entries.
    NULL,

    /// The array element specifies a loadable segment, described by `filesz`
    /// and `memsz`. The bytes from the file are mapped to the beginning of the
    /// memory segment. If the segment's memory size `memsz` is larger than the
    /// file size `filesz`, the "extra" bytes are defined to hold the value 0
    /// and to follow the segment's initialized area. The file size may not be
    /// larger than the memory size. Loadable segment entries in the program
    /// header table appear in ascending order, sorted on the `vaddr` member.
    LOAD,

    /// The array element specifies dynamic linking information.
    DYNAMIC,

    /// The array element specifies the location and size of a null-terminated
    /// pathname to invoke as an interpreter. This segment type is meaningful
    /// only for executable files (though  it may occur for shared objects).
    /// However it may not occur more than once in a file. If it is present, it
    /// must precede any loadable segment entry.
    INTERP,

    /// The array element specifies the location and size for auxiliary
    /// information.
    NOTE,

    /// This segment type is reserved but has unspecified semantics. Programs
    /// that contain an array element of this type do not conform to the ABI.
    SHLIB,

    /// The array element, if present, specifies the location and size of the
    /// program header table itself, both in the file and in the memory image of
    /// the program. This segment type may not occur more than once in a file.
    /// Moreover, it may occur only if the program header table is part of the
    /// memory image of the program. If it is present, it must precede any
    /// loadable segment entry.
    PHDR,

    /// TODO: Man page doesn't list this, but header files has it.
    TLS,

    /// Reserved for processor-specific semantics.
    OS(u32),

    /// Reserved for processor-specific semantics.
    PROC(u32),

    /// GNU extension which is used by the Linux kernel to control the state of
    /// the stack via the flags set in the `flags` member.
    GNU_EH_FRAME,
}

#[derive(Debug)]
pub enum SectionHeader { SectionHeader32(SectionHeader32), SectionHeader64(SectionHeader64) }

#[derive(Debug)]
pub struct SectionHeader32 {
    /// Name of the section. Its value is an index into the section header
    /// string table section, giving the location of a null-terminated string.
    pub name: u32,

    /// Categorizes the section's contents and semantics.
    pub ty: SectionHeaderType,

    pub flags: u32,

    /// If the section will appear in the memory image of a process, this is the
    /// address at which the section's first byte should reside. Otherwise it's 0.
    pub addr: u32,

    /// The byte offset from the beginning of the file to the first byte in the
    /// section.
    pub offset: u32,

    /// The section's size in bytes.
    pub size: u32,

    /// Section header table index link.
    // ???
    pub link: u32,

    /// Extra information, whose interpretation depends on the section type.
    pub info: u32,

    /// Alignment constraints.
    pub addralign: u32,

    /// Some sections hold a table of fixed-size entries, such as a symbol
    /// table. For such a section, this gives the size in bytes of each entry.
    /// 0 if the section does not hold a table of fixed-size entries.
    pub entsize: u32,
}

/// See documentation of `SectionHeader64`.
#[derive(Debug, Clone)]
pub struct SectionHeader64 {
    pub name: u32,
    pub ty: SectionHeaderType,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addralign: u64,
    pub entsize: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum SectionHeaderType {
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

    // FIXME: This is not in the man page, but header file has it.
    NUM,

    /// This value up to and including `HIPROC` is reserved for
    /// processor-specific semantics.
    LOPROC,

    /// A value between `LOPROC` and `HIPROC`.
    PROC(u32),

    /// This value down to and including `LOPROC` is reserved for
    /// processor-specific semantics.
    HIPROC,

    /// This value specifies the lower bound of the range of indices reserved
    /// for application programs.
    LOUSER,

    /// A value between `LOUSER` and `HIUSER`.
    USER(u32),

    /// This value specifies the upper bound of the range of indices reserved
    /// for application programs. Section types between `LOUSER` and `HIUSER`
    /// may be used by the application, without conflicting with current or
    /// future system-defined section types.
    HIUSER,

    // (Found in the wild)
    GNU_HASH, VERSYM, VERNEED, INIT_ARRAY, FINI_ARRAY,
}

////////////////////////////////////////////////////////////////////////////////
// Parsing
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ParseResult {
    ParseOK(ELFHeader),
    NotELF,
    CantReadFile(Error),
    CantParse,
}

pub fn parse_elf_header(path : &Path) -> ParseResult {
    let mut contents = Vec::new();

    match File::open(path) {
        Err(err) => ParseResult::CantReadFile(err),
        Ok(mut file) => {
            file.read_to_end(&mut contents);
            parse_elf_header_(contents.borrow())
        }
    }
}

pub fn parse_elf_header_(contents : &[u8]) -> ParseResult {
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
        match read_u16(endianness, &contents[ 0x10 .. ]) {
            1 => ObjType::Relocatable,
            2 => ObjType::Executable,
            3 => ObjType::Shared,
            4 => ObjType::Core,
            _ => return ParseResult::CantParse,
        };

    let isa =
        match read_u16(endianness, &contents[ 0x12 .. ]) {
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
                read_u32(endianness, &contents[ 0x18 .. ]) as u64
            },
            Class::Bit64 => {
                read_u64(endianness, &contents[ 0x18 .. ])
            }
        };

    let phoff =
        match class {
            Class::Bit32 => {
                read_u32(endianness, &contents[ 0x1C .. ]) as u64
            },
            Class::Bit64 => {
                read_u64(endianness, &contents[ 0x20 .. ])
            }
        };

    let shoff =
        match class {
            Class::Bit32 => {
                read_u32(endianness, &contents[ 0x20 .. ]) as u64
            },
            Class::Bit64 => {
                read_u64(endianness, &contents[ 0x28 .. ])
            }
        };

    let flags =
        match class {
            Class::Bit32 => {
                read_u32(endianness, &contents[ 0x24 .. ])
            },
            Class::Bit64 => {
                read_u32(endianness, &contents[ 0x30 .. ])
            }
        };

    let ehsize =
        match class {
            Class::Bit32 => {
                read_u16(endianness, &contents[ 0x28 .. ])
            },
            Class::Bit64 => {
                read_u16(endianness, &contents[ 0x34 .. ])
            }
        };

    let phentsize =
        match class {
            Class::Bit32 => {
                read_u16(endianness, &contents[ 0x2A .. ])
            },
            Class::Bit64 => {
                read_u16(endianness, &contents[ 0x36 .. ])
            }
        };

    let phnum =
        match class {
            Class::Bit32 => {
                read_u16(endianness, &contents[ 0x2C .. ])
            },
            Class::Bit64 => {
                read_u16(endianness, &contents[ 0x38 .. ])
            }
        };

    let shentsize =
        match class {
            Class::Bit32 => {
                read_u16(endianness, &contents[ 0x2E .. ])
            },
            Class::Bit64 => {
                read_u16(endianness, &contents[ 0x3A .. ])
            }
        };

    let shnum =
        match class {
            Class::Bit32 => {
                read_u16(endianness, &contents[ 0x30 .. ])
            },
            Class::Bit64 => {
                read_u16(endianness, &contents[ 0x3C .. ])
            }
        };

    let shstrndx =
        match class {
            Class::Bit32 => {
                read_u16(endianness, &contents[ 0x32 .. ])
            },
            Class::Bit64 => {
                read_u16(endianness, &contents[ 0x3E .. ])
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

pub fn parse_program_headers(elf_header : &ELFHeader, contents: &[u8]) -> Vec<ProgramHeader> {
    let num_pgm_headers      = elf_header.phnum as usize;
    let pgm_header_size      = elf_header.phentsize as usize;
    let pgm_headers_start_at = elf_header.phoff as usize;

    let class                = elf_header.class;
    let endianness           = elf_header.endianness;

    let mut ret = Vec::new();

    for i in 0 .. num_pgm_headers {
        let start_offset = pgm_headers_start_at + i * pgm_header_size;
        let header_bits  = &contents[ start_offset .. ];

        let header = match class {
            Class::Bit32 =>
                ProgramHeader::ProgramHeader32(
                    parse_program_header_32(endianness, header_bits)),
            Class::Bit64 =>
                ProgramHeader::ProgramHeader64(
                    parse_program_header_64(endianness, header_bits)),
        };

        ret.push(header);
    }

    ret
}

fn parse_program_header_32(endianness : Endianness, contents: &[u8]) -> ProgramHeader32 {
    let ty     = read_u32(endianness,  contents);
    let offset = read_u32(endianness, &contents[  4 .. ]);
    let vaddr  = read_u32(endianness, &contents[  8 .. ]);
    let paddr  = read_u32(endianness, &contents[ 12 .. ]);
    let filesz = read_u32(endianness, &contents[ 16 .. ]);
    let memsz  = read_u32(endianness, &contents[ 20 .. ]);
    let flags  = read_u32(endianness, &contents[ 24 .. ]);
    let align  = read_u32(endianness, &contents[ 30 .. ]);

    ProgramHeader32 {
        ty: parse_program_header_ty(ty),
        offset: offset,
        vaddr: vaddr,
        paddr: paddr,
        filesz: filesz,
        memsz: memsz,
        flags: flags,
        align: align,
    }
}

fn parse_program_header_64(endianness : Endianness, contents: &[u8]) -> ProgramHeader64 {
    let ty     = read_u32(endianness,  contents);
    let flags  = read_u32(endianness, &contents[  4 .. ]);
    let offset = read_u64(endianness, &contents[  8 .. ]);
    let vaddr  = read_u64(endianness, &contents[ 16 .. ]);
    let paddr  = read_u64(endianness, &contents[ 24 .. ]);
    let filesz = read_u64(endianness, &contents[ 32 .. ]);
    let memsz  = read_u64(endianness, &contents[ 40 .. ]);
    let align  = read_u64(endianness, &contents[ 48 .. ]);

    ProgramHeader64 {
        ty: parse_program_header_ty(ty),
        flags: flags,
        offset: offset,
        vaddr: vaddr,
        paddr: paddr,
        filesz: filesz,
        memsz: memsz,
        align: align,
    }
}

fn parse_program_header_ty(ty : u32) -> ProgramHeaderType {
    match ty {
        0 => ProgramHeaderType::NULL,
        1 => ProgramHeaderType::LOAD,
        2 => ProgramHeaderType::DYNAMIC,
        3 => ProgramHeaderType::INTERP,
        4 => ProgramHeaderType::NOTE,
        5 => ProgramHeaderType::SHLIB,
        6 => ProgramHeaderType::PHDR,
        7 => ProgramHeaderType::TLS,
        0x6474e550 => ProgramHeaderType::GNU_EH_FRAME,
        0x60000000 ... 0x6fffffff => ProgramHeaderType::OS(ty),
        0x70000000 ... 0x7fffffff => ProgramHeaderType::PROC(ty),
        _ => panic!("parse_program_header_ty: Unknown program header type: 0x{0:X}", ty),
    }
}

pub fn parse_section_headers(elf_header : &ELFHeader, contents: &[u8]) -> Vec<SectionHeader> {
    let num_section_headers = elf_header.shnum as usize;
    let section_header_size = elf_header.shentsize as usize;
    let headers_start_at    = elf_header.shoff as usize;

    let class               = elf_header.class;
    let endianness          = elf_header.endianness;

    let mut ret = Vec::new();

    for i in 0 .. num_section_headers {
        let start_offset = headers_start_at + i * section_header_size;
        let header_bits  = &contents[ start_offset .. ];

        let header = match class {
            Class::Bit32 =>
                SectionHeader::SectionHeader32(
                    parse_section_header_32(endianness, header_bits)),
            Class::Bit64 =>
                SectionHeader::SectionHeader64(
                    parse_section_header_64(endianness, header_bits))
        };

        ret.push(header);
    }

    ret
}

fn header_ty(h : &SectionHeader) -> SectionHeaderType {
    match *h {
        SectionHeader::SectionHeader32(ref h) => h.ty,
        SectionHeader::SectionHeader64(ref h) => h.ty,
    }
}

fn parse_section_header_32(endianness : Endianness, contents : &[u8]) -> SectionHeader32 {
    let name   = read_u32(endianness,  contents);
    let ty     = read_u32(endianness, &contents[ 4 .. ]);
    let flags  = read_u32(endianness, &contents[ 8 .. ]);
    let addr   = read_u32(endianness, &contents[ 12 .. ]);
    let offset = read_u32(endianness, &contents[ 16 .. ]);
    let size   = read_u32(endianness, &contents[ 20 .. ]);
    let link   = read_u32(endianness, &contents[ 24 .. ]);
    let info   = read_u32(endianness, &contents[ 28 .. ]);
    let addralign = read_u32(endianness, &contents[ 32 .. ]);
    let entsize = read_u32(endianness, &contents[ 36 .. ]);

    SectionHeader32 {
        name: name,
        ty: parse_section_header_ty(ty),
        flags: flags,
        addr: addr,
        offset: offset,
        size: size,
        link: link,
        info: info,
        addralign: addralign,
        entsize: entsize,
    }
}

fn parse_section_header_64(endianness : Endianness, contents : &[u8]) -> SectionHeader64 {
    let name   = read_u32(endianness,  contents);
    let ty     = read_u32(endianness, &contents[ 4 .. ]);
    let flags  = read_u64(endianness, &contents[ 8 .. ]);
    let addr   = read_u64(endianness, &contents[ 16 .. ]);
    let offset = read_u64(endianness, &contents[ 24 .. ]);
    let size   = read_u64(endianness, &contents[ 32 .. ]);
    let link   = read_u32(endianness, &contents[ 40 .. ]);
    let info   = read_u32(endianness, &contents[ 44 .. ]);
    let addralign = read_u64(endianness, &contents[ 48 .. ]);
    let entsize = read_u64(endianness, &contents[ 56 .. ]);

    SectionHeader64 {
        name: name,
        ty: parse_section_header_ty(ty),
        flags: flags,
        addr: addr,
        offset: offset,
        size: size,
        link: link,
        info: info,
        addralign: addralign,
        entsize: entsize,
    }
}

fn parse_section_header_ty(ty : u32) -> SectionHeaderType {
    match ty {
         0 => SectionHeaderType::NULL,
         1 => SectionHeaderType::PROGBITS,
         2 => SectionHeaderType::SYMTAB,
         3 => SectionHeaderType::STRTAB,
         4 => SectionHeaderType::RELA,
         5 => SectionHeaderType::HASH,
         6 => SectionHeaderType::DYNAMIC,
         7 => SectionHeaderType::NOTE,
         8 => SectionHeaderType::NOBITS,
         9 => SectionHeaderType::REL,
        10 => SectionHeaderType::SHLIB,
        11 => SectionHeaderType::DYNSYM,
        12 => SectionHeaderType::NUM,
        0x70000000 ... 0x7fffffff => SectionHeaderType::PROC(ty),
        0x80000000 ... 0xffffffff => SectionHeaderType::USER(ty),

        // Some types found in the wild
        0x6ffffff6 => SectionHeaderType::GNU_HASH,
        0x6fffffff => SectionHeaderType::VERSYM,
        0x6ffffffe => SectionHeaderType::VERNEED,
        0xe        => SectionHeaderType::INIT_ARRAY,
        0xf        => SectionHeaderType::FINI_ARRAY,

        _ => panic!("parse_section_header_type: Unknown section header type: 0x{0:X}", ty),
    }
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
            ((read_u16(endianness, &from[ 2 ..  ]) as u32) << 16)
                | (read_u16(endianness, from) as u32)
        },
        Endianness::BigEndian => {
            ((read_u16(endianness, from) as u32) << 16)
                | (read_u16(endianness, &from[ 2 .. ]) as u32)
        }
    }
}

fn read_u64(endianness : Endianness, from : &[u8]) -> u64 {
    match endianness {
        Endianness::LittleEndian => {
            ((read_u32(endianness, &from[ 4 .. ]) as u64) << 32)
                | (read_u32(endianness, from) as u64)
        },
        Endianness::BigEndian => {
            ((read_u32(endianness, from) as u64) << 32)
                | (read_u32(endianness, &from[ 4 .. ]) as u64)
        }
    }
}

/// Extract bytes of string slices from a string table.
/// Strings do not have null terminator!
pub fn read_string_table(bytes : &[u8]) -> Vec<&[u8]> {
    let mut ret = Vec::new();

    // First byte is always zero, as is the last byte.
    let mut start = 1;
    let mut i     = 2;

    while i < bytes.len() {
        if unsafe { *bytes.get_unchecked(i) } == 0 {
            ret.push( &bytes[ start .. i ] );
            start = i + 1;
            i     = start + 1;
        } else {
            i += 1;
        }
    }

    ret
}

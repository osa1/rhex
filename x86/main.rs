#![feature(str_char)]

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;
use std::io::BufRead;
use std::fmt::Debug;

pub mod instr_table;

////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum Reg64 {
    RAX = 0, RCX, RDX, RBX, RSP, RBP, RSI, RDI,
    R8, R9, R10, R11, R12, R13, R14, R15
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum Reg32 {
    EAX = 0, ECX, EDX, EBX, ESP, EBP, ESI, EDI,
    R8D, R9D, R10D, R11D, R12D, R13D, R14D, R15D
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum Reg16 {
    AX = 0, CX, DX, BX, SP, BP, SI, DI,
    R8W, R9W, R10W, R11W, R12W, R13W, R14W, R15W
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum Reg8 {
    AL = 0, CL, DL, BL, AH, CH, DH, BH, SPL, BPL, SIL, DIL,
    R8L, R9L, R10L, R11L, R12L, R13L, R14L, R15L
}

type Mem64 = Reg64;
type Mem32 = Reg32;
type Mem16 = Reg16;
type Mem8  = Reg8;

type Imm8  = u8;
type Imm16 = u16;
type Imm32 = u32;
type Imm64 = u64;

#[derive(Debug)]
enum Reg {
    Reg8(Reg8),
    Reg16(Reg16),
    Reg32(Reg32),
    Reg64(Reg64),
}

////////////////////////////////////////////////////////////////////////////////

trait Instr : Debug {
    fn encode(&self, buffer : &mut Vec<u8>);
}

////////////////////////////////////////////////////////////////////////////////

fn rex_pfx(w : bool, r : bool, x : bool, b : bool) -> u8 {
    let mut ret = 0b0100_0000;

    if w { ret |= 0b1000; }
    if r { ret |= 0b0100; }
    if x { ret |= 0b0010; }
    if b { ret |= 0b0001; }

    ret
}

static REX_W : u8 = 0b0100_1000;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Add_RM64_R64 {
    pub op1 : Reg64, // TODO: This can be a memory location
    pub op2 : Reg64,
}

impl Instr for Add_RM64_R64 {
    fn encode(&self, buffer : &mut Vec<u8>) {
        // REX.W
        let mut rexw = REX_W;
        if self.op2 >= Reg64::R8 { rexw |= 0b0000_0100; }
        if self.op1 >= Reg64::R8 { rexw |= 0b0000_0001; }
        buffer.push(rexw);
        // opcode
        buffer.push(0x01);
        // ModR/M
        buffer.push(0b1100_0000
                    | (((self.op2 as u8) << 3) & 0b00111000)
                    | ((self.op1 as u8) & 0b00000111));
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Add_RM32_IB {
    pub op1 : Reg32, // TODO: This can be a memory location
    pub op2 : Imm8,
}

impl Instr for Add_RM32_IB {
    fn encode(&self, buffer : &mut Vec<u8>) {
        // opcode
        buffer.push(0x83);
        // ModR/M
        buffer.push(0b1100_0000 | ((self.op1 as u8) << 3));
        // immediate byte
        buffer.push(self.op2);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Ret {}

impl Instr for Ret {
    fn encode(&self, buffer : &mut Vec<u8>) {
        // opcode
        buffer.push(0xC3);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Mov_R64_R64 {
    pub op1 : Reg64,
    pub op2 : Reg64
}

impl Instr for Mov_R64_R64 {
    fn encode(&self, buffer : &mut Vec<u8>) {
        // REX
        let mut rexw = REX_W;
        if self.op2 >= Reg64::R8 { rexw |= 0b0000_0100; }
        if self.op1 >= Reg64::R8 { rexw |= 0b0000_0001; }
        buffer.push(rexw);
        // opcode
        buffer.push(0x89);
        // ModR/M
        buffer.push(0b1100_0000 // Reg-Reg
                    | (((self.op2 as u8) << 3) & 0b00111000)
                    | ((self.op1 as u8) & 0b00000111));
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Mov_M64_R64 {
    pub op1 : Reg64, // Address pointed by reg
    pub op2 : Reg64
}

impl Instr for Mov_M64_R64 {
    fn encode(&self, buffer : &mut Vec<u8>) {
        // REX
        let mut rexw = REX_W;
        if self.op2 >= Reg64::R8 { rexw |= 0b0000_0100; }
        if self.op1 >= Reg64::R8 { rexw |= 0b0000_0001; }
        buffer.push(rexw);
        // opcode
        buffer.push(0x89);
        // ModR/M
        buffer.push(0b0000_0000 // Mem-Reg
                    | (((self.op2 as u8) << 3) & 0b00111000)
                    | ((self.op1 as u8) & 0b00000111));
    }
}

////////////////////////////////////////////////////////////////////////////////

fn to_hex_string(bytes : &Vec<u8>) -> String {
    let strs : Vec<String> = bytes.iter()
                                  .map(|b| format!("{:02X}", b))
                                  .collect();
    strs.join(" ")
}

fn encode_and_print(instr : Box<Instr>) {
    let mut buf : Vec<u8> = Vec::with_capacity(3);
    instr.encode(&mut buf);
    println!("{:?}: {}", instr, to_hex_string(&buf));
}

////////////////////////////////////////////////////////////////////////////////

type Opcode = Vec<OpcodePart>;

#[derive(Debug)]
enum OpcodePart {
    /// Just a byte
    Byte(u8),

    /// Byte+i, see +i in 3.1.1.1
    BytePlusI(u8),

    /// Byte+r, see +r in 3.1.1.1
    BytePlusReg(u8),

    /// REX.W sequence (0b0100_1000)
    ///                        ^ w
    REXW,

    /// Immediate byte
    IB,

    /// Immediate word
    IW,

    /// Immediate double word
    ID,

    /// Immediate quad word
    IQ,

    /// A digit that specifies a colum in Table 2-1 and 2.2 (ModR/M encoding)
    /// (see Section 3.1.1.1)
    Digit(u8),

    ModRM,
}

fn parse_opcode_part(s : &str) -> Option<OpcodePart> {
    if s == "REX.W" {
        Some(OpcodePart::REXW)
    } else if s == "/r" {
        Some(OpcodePart::ModRM)
    } else if s.char_at(0) == '/' {
        match (&s[1..]).parse::<u8>() {
            Err(err) => { panic!("parse_opcode_part: Can't parse {} {}", s, err); }
            Ok(d) => Some(OpcodePart::Digit(d))
        }
    } else if s == "ib" {
        Some(OpcodePart::IB)
    } else if s == "iw" {
        Some(OpcodePart::IW)
    } else if s == "id" {
        Some(OpcodePart::ID)
    } else if s == "iq" {
        Some(OpcodePart::IQ)
    }

    // Ignoring these for now
    else if s.starts_with("VEX") || s.starts_with("EVEX") || s.starts_with("XOP") {
        None
    }

    else {
        let hex_digit_1 = s.char_at(0).to_digit(16).unwrap_or_else(|| panic!("Not valid hex: {}", s));
        let hex_digit_2 = s.char_at(1).to_digit(16).unwrap_or_else(|| panic!("Not valid hex: {}", s));
        let byte = (hex_digit_1 * 16 + hex_digit_2) as u8;
        if s.len() > 2 {
            if &s[2..] == "+i" {
                Some(OpcodePart::BytePlusI(byte))
            } else if &s[2..] == "+r" {
                Some(OpcodePart::BytePlusReg(byte))
            } else {
                panic!("Can't parse opcode part: {}", s)
            }
        } else {
            Some(OpcodePart::Byte(byte))
        }
    }
}

fn parse_opcode(s : &str) -> Option<Opcode> {
    s.split(' ').map(parse_opcode_part).collect::<Option<Opcode>>()
}

#[derive(Debug)]
enum Operand {
    Implicit(Reg),
    Explicit(Reg),

    R8, R16, R32, R64,

    M8, M16, M32, M64, M128,

    R8_M8, R16_M16, R32_M32, R64_M64,

    MEM,

    MOFF8, MOFF16, MOFF32, MOFF64,

    /// Segment register
    SREG,

    /// R16, R32 or R64
    RXX,

    /// M16, M32 or M64
    MXX,

    /// R16, R32, R64, M16, M32 or M64
    RXX_MXX,

    IB, IW, ID, IQ,

    Rel8, Rel16, Rel32,

    /// es:zdi
    EXP_ES_ZDI,
    IMP_ES_ZDI,

    /// ds:zdi
    EXP_DS_ZDI,
    IMP_DS_ZDI,

    /// ds:zsi
    EXP_DS_ZSI,
    IMP_DS_ZSI,

    CS, DS, ES, SS, FS, GS,

    One,
}

fn parse_operand(s : &str) -> Option<Operand> {
    // TODO: Maybe move these into a table
    if      s == ""  { None }

    else if s == "1" { Some(Operand::One) }

    else if s == "<al>"  { Some(Operand::Implicit(Reg::Reg8 (Reg8::AL )))  }
    else if s == "<ah>"  { Some(Operand::Implicit(Reg::Reg8 (Reg8::AH )))  }
    else if s == "<ax>"  { Some(Operand::Implicit(Reg::Reg16(Reg16::AX)))  }
    else if s == "<dx>"  { Some(Operand::Implicit(Reg::Reg16(Reg16::DX)))  }
    else if s == "<eax>" { Some(Operand::Implicit(Reg::Reg32(Reg32::EAX))) }
    else if s == "<ebx>" { Some(Operand::Implicit(Reg::Reg32(Reg32::EBX))) }
    else if s == "<ecx>" { Some(Operand::Implicit(Reg::Reg32(Reg32::ECX))) }
    else if s == "<edx>" { Some(Operand::Implicit(Reg::Reg32(Reg32::EDX))) }
    else if s == "<rax>" { Some(Operand::Implicit(Reg::Reg64(Reg64::RAX))) }
    else if s == "<rbx>" { Some(Operand::Implicit(Reg::Reg64(Reg64::RBX))) }
    else if s == "<rcx>" { Some(Operand::Implicit(Reg::Reg64(Reg64::RCX))) }
    else if s == "<rdx>" { Some(Operand::Implicit(Reg::Reg64(Reg64::RDX))) }

    else if s == "cs"  { Some(Operand::CS) }
    else if s == "ds"  { Some(Operand::DS) }
    else if s == "es"  { Some(Operand::ES) }
    else if s == "ss"  { Some(Operand::SS) }
    else if s == "fs"  { Some(Operand::FS) }
    else if s == "gs"  { Some(Operand::GS) }

    else if s == "al"  { Some(Operand::Explicit(Reg::Reg8 (Reg8::AL  ))) }
    else if s == "cl"  { Some(Operand::Explicit(Reg::Reg8 (Reg8::CL )))  }
    else if s == "ax"  { Some(Operand::Explicit(Reg::Reg16(Reg16::AX ))) }
    else if s == "cx"  { Some(Operand::Explicit(Reg::Reg16(Reg16::CX ))) }
    else if s == "dx"  { Some(Operand::Explicit(Reg::Reg16(Reg16::DX ))) }
    else if s == "eax" { Some(Operand::Explicit(Reg::Reg32(Reg32::EAX))) }
    else if s == "ecx" { Some(Operand::Explicit(Reg::Reg32(Reg32::ECX))) }
    else if s == "edx" { Some(Operand::Explicit(Reg::Reg32(Reg32::EDX))) }
    else if s == "rax" { Some(Operand::Explicit(Reg::Reg64(Reg64::RAX))) }
    else if s == "rcx" { Some(Operand::Explicit(Reg::Reg64(Reg64::RCX))) }
    else if s == "rdx" { Some(Operand::Explicit(Reg::Reg64(Reg64::RDX))) }

    else if s == "r8"  { Some(Operand::R8 ) }
    else if s == "r16" { Some(Operand::R16) }
    else if s == "r32" { Some(Operand::R32) }
    else if s == "r64" { Some(Operand::R64) }

    else if s == "m8"   { Some(Operand::M8  ) }
    else if s == "m16"  { Some(Operand::M16 ) }
    else if s == "m32"  { Some(Operand::M32 ) }
    else if s == "m64"  { Some(Operand::M64 ) }
    else if s == "m128" { Some(Operand::M128) }

    else if s == "ib" { Some(Operand::IB) }
    else if s == "iw" { Some(Operand::IW) }
    else if s == "id" { Some(Operand::ID) }
    else if s == "iq" { Some(Operand::IQ) }

    else if s == "r8/m8"   { Some(Operand::R8_M8  ) }
    else if s == "r16/m16" { Some(Operand::R16_M16) }
    else if s == "r32/m32" { Some(Operand::R32_M32) }
    else if s == "r64/m64" { Some(Operand::R64_M64) }

    else if s == "rel8"  { Some(Operand::Rel8 ) }
    else if s == "rel16" { Some(Operand::Rel16) }
    else if s == "rel32" { Some(Operand::Rel32) }

    else if s == "rxx"     { Some(Operand::RXX)     }
    else if s == "mxx"     { Some(Operand::MXX)     }
    else if s == "rxx/mxx" { Some(Operand::RXX_MXX) }

    else if s == "<es:zdi>" { Some(Operand::IMP_ES_ZDI) }
    else if s == "es:zdi"   { Some(Operand::EXP_ES_ZDI) }

    else if s == "<ds:zdi>" { Some(Operand::IMP_DS_ZDI) }
    else if s == "ds:zdi"   { Some(Operand::EXP_DS_ZDI) }

    else if s == "<ds:zsi>" { Some(Operand::IMP_DS_ZSI) }
    else if s == "ds:zsi"   { Some(Operand::EXP_DS_ZSI) }

    // lea etc.
    else if s == "mem"   { Some(Operand::MEM) }

    else if s == "sreg"   { Some(Operand::SREG) }

    else if s == "moff8"    { Some(Operand::MOFF8) }
    else if s == "moff16"   { Some(Operand::MOFF16) }
    else if s == "moff32"   { Some(Operand::MOFF32) }
    else if s == "moff64"   { Some(Operand::MOFF64) }

    // TODO: Dropping some prefix for now
    else if s.len() > 2 {
        let pfx = &s[ .. 2 ];
        if pfx == "r:" {
            parse_operand(&s[ 2 .. ])
        } else if pfx == "w:" {
            parse_operand(&s[ 2 .. ])
        } else if pfx == "x:" {
            parse_operand(&s[ 2 .. ])
        } else if pfx == "R:" {
            parse_operand(&s[ 2 .. ])
        } else if pfx == "W:" {
            parse_operand(&s[ 2 .. ])
        } else if pfx == "X:" {
            parse_operand(&s[ 2 .. ])
        } else {
            // panic!("Can't parse operand: {}", s)
            // ignore for now
            None
        }
    } else {
        // panic!("Can't parse operand: {}", s)
        // ignore for now
        None
    }
}

fn parse_operands(s : &str) -> Option<Vec<Operand>> {
    s.split(' ').map(|s| parse_operand(s.trim().trim_matches(','))).collect::<Option<Vec<Operand>>>()
}

#[derive(Debug)]
struct Instr_ {
    pub mnem : &'static str,
    pub operands : Vec<Operand>,
    // pub encoding : &'static str,
    pub opcode : Opcode,
}

fn main() {
    // let mut buf : Vec<u8> = Vec::with_capacity(3);
    // let instr = Box::new(Add_RM64_R64 { op1 : Reg64::RAX, op2 : Reg64::RCX });
    // encode_and_print(instr);

    // let instr = Box::new(Add_RM64_R64 { op1 : Reg64::R13, op2 : Reg64::R15 });
    // encode_and_print(instr);

    // let instr = Box::new(Add_RM32_IB { op1 : Reg32::EAX, op2 : 12 });
    // encode_and_print(instr);

    // let instr = Box::new(Ret{});
    // encode_and_print(instr);

    // let instr = Box::new(Mov_R64_R64 { op1 : Reg64::RCX, op2 : Reg64::R15 });
    // encode_and_print(instr);

    // let instr = Box::new(Mov_M64_R64 { op1 : Reg64::RAX, op2 : Reg64::R9 });
    // encode_and_print(instr);
    let mut instrs = Vec::new();
    for instr in instr_table::INSTR_STRS.iter() {
        let mnem = instr[0];
        let operands = instr[1];
        let encoding = instr[2];
        let opcodes = instr[3];
        let tags = instr[4];

        if let Some(opcode) = parse_opcode(opcodes) {
            if let Some(operands) = parse_operands(operands) {
                instrs.push(Instr_ {
                    mnem : mnem,
                    operands : operands,
                    // encoding : panic!(),
                    opcode : opcode
                });
            }
        }
    }

    println!("{} instructions parsed.", instrs.len());

    for instr in instrs {
        println!("{:?}", instr);
    }
}

#![feature(str_char)]

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;
use std::io::BufRead;
use std::fmt::Debug;
use std::ffi::CString;


extern crate libc;

// pub mod instr_table;

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
    AL = 0, CL, DL, BL, AH, CH, DH, BH,
    SPL, BPL, SIL, DIL,
    R8L, R9L, R10L, R11L, R12L, R13L, R14L, R15L
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum XMM {
    XMM0 = 0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
    XMM8, XMM9, XMM10, XMM11, XMM12, XMM13, XMM14, XMM15
}

#[inline]
fn reg16_bits(reg : Reg16) -> (bool, u8) {
    if reg >= Reg16::R8W {
        (true, (reg as u8) & 0b0000_0111)
    } else {
        (false, reg as u8)
    }
}

#[inline]
fn reg32_bits(reg : Reg32) -> (bool, u8) {
    if reg >= Reg32::R8D {
        (true, (reg as u8) & 0b0000_0111)
    } else {
        (false, reg as u8)
    }
}

#[inline]
fn reg64_bits(reg : Reg64) -> (bool, u8) {
    if reg >= Reg64::R8 {
        (true, (reg as u8) & 0b0000_0111)
    } else {
        (false, reg as u8)
    }
}

#[inline]
fn xmm_bits(xmm : XMM) -> (bool, u8) {
    if xmm >= XMM::XMM8 {
        (true, (xmm as u8) & 0b0000_0111)
    } else {
        (false, xmm as u8)
    }
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

type Mem = Reg;

#[inline]
fn encode_reg_modrm(reg : Reg) -> u8 {
    match reg {
        Reg::Reg8 (reg8 ) => reg8 as u8,
        Reg::Reg16(reg16) => reg16 as u8,
        Reg::Reg32(reg32) => reg32 as u8,
        Reg::Reg64(reg64) => reg64 as u8,
    }
}

struct Disp8_Reg32 {
    reg : Reg32,
    disp : u8,
}

struct Disp8_Reg64 {
    reg : Reg64,
    disp : u8,
}

struct Disp32_Reg32 {
    reg : Reg32,
    disp : u32,
}

struct Disp32_Reg64 {
    reg : Reg64,
    disp : u32,
}


////////////////////////////////////////////////////////////////////////////////

trait Instr : Debug {
    fn encode(&self, buffer : &mut Vec<u8>);
}

////////////////////////////////////////////////////////////////////////////////

#[inline]
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

////////////////////////////////////////////////////////////////////////////////

#[inline]
fn encode_u16(iw : u16, buf : &mut Vec<u8>) {
    buf.push(iw as u8);
    buf.push((iw > 8) as u8);
}

#[inline]
fn encode_u32(id : u32, buf : &mut Vec<u8>) {
    buf.push(id as u8);
    buf.push((id >> 8) as u8);
    buf.push((id >> 16) as u8);
    buf.push((id >> 24) as u8);
}

#[inline]
fn encode_u64(iq : u64, buf : &mut Vec<u8>) {
    buf.push(iq as u8);
    buf.push((iq >> 8) as u8);
    buf.push((iq >> 16) as u8);
    buf.push((iq >> 24) as u8);
    buf.push((iq >> 32) as u8);
    buf.push((iq >> 40) as u8);
    buf.push((iq >> 48) as u8);
    buf.push((iq >> 54) as u8);
}

////////////////////////////////////////////////////////////////////////////////

fn encode_aax(buf : &mut Vec<u8>) {
    buf.push(0x37);
}

fn encode_aas(buf : &mut Vec<u8>) {
    buf.push(0x3F);
}

fn encode_aad(ib : u8, buf : &mut Vec<u8>) {
    buf.push(0xD5);
    buf.push(ib);
}

fn encode_aam(ib : u8, buf : &mut Vec<u8>) {
    buf.push(0xD4);
    buf.push(ib);
}

fn encode_adc_al_ib(ib : u8, buf : &mut Vec<u8>) {
    buf.push(0x14);
    buf.push(ib);
}

fn encode_adc_ax_iw(iw : u16, buf : &mut Vec<u8>) {
    buf.push(0x66);
    buf.push(0x15);
    encode_u16(iw, buf);
}

fn encode_adc_eax_id(id : u32, buf : &mut Vec<u8>) {
    buf.push(0x15);
    encode_u32(id, buf);
}

fn encode_adc_rax_id(id : u32, buf : &mut Vec<u8>) {
    buf.push(REX_W);
    buf.push(0x15);
    encode_u32(id, buf);
}

fn encode_adc_m64_r64(m64 : Mem64, r64 : Reg64, buf : &mut Vec<u8>) {
    let mut rexw = REX_W;
    if r64 >= Reg64::R8 { rexw |= 0b0000_0100; }
    if m64 >= Reg64::R8 { rexw |= 0b0000_0001; }
    buf.push(rexw);
    buf.push(0x11);
    // Mod: Mem - Reg = 00
    // MR
    buf.push((((r64 as u8) & 0b00000111) << 3) | m64 as u8);
}

fn encode_lea_r64_mem(r64 : Reg64, mem : Mem, buf : &mut Vec<u8>) {
    let mut rexw = REX_W;
    if r64 >= Reg64::R8 {
        rexw |= 0b0000_0100;
    }
    if let Reg::Reg64(reg64) = mem {
        if reg64 >= Reg64::R8 {
            rexw |= 0b0000_0001;
        }
    };
    buf.push(rexw);
    buf.push(0x8D);
    // mod = 0
    buf.push((((r64 as u8) & 0b00000111) << 3) | encode_reg_modrm(mem));
}

fn encode_lea_r64_disp8_32(r64 : Reg64, disp : Disp8_Reg32, buf : &mut Vec<u8>) {
    // Need address-size override prefix
    buf.push(0x67);
    let mut rexw = REX_W;
    if r64 >= Reg64::R8 {
        rexw |= 0b0000_0100;
    }
    buf.push(rexw);
    buf.push(0x8D);
    // mod = 01
    buf.push(0b0100_0000 | (((r64 as u8) & 0b00000111) << 3) | (disp.reg as u8));
    buf.push(disp.disp);
}

fn encode_lea_r64_disp8_64(r64 : Reg64, disp : Disp8_Reg64, buf : &mut Vec<u8>) {
    let mut rexw = REX_W;
    if r64 >= Reg64::R8 {
        rexw |= 0b0000_0100;
    }
    if disp.reg >= Reg64::R8 {
        rexw |= 0b0000_0001;
    }
    buf.push(rexw);
    buf.push(0x8D);
    // mod = 01
    buf.push(0b0100_0000 | (((r64 as u8) & 0b00000111) << 3) | (disp.reg as u8));
    buf.push(disp.disp);
}

fn encode_lea_r64_disp32_32(r64 : Reg64, disp : Disp32_Reg32, buf : &mut Vec<u8>) {
    // Need address-size override prefix
    buf.push(0x67);
    let mut rexw = REX_W;
    if r64 >= Reg64::R8 {
        rexw |= 0b0000_0100;
    }
    buf.push(rexw);
    buf.push(0x8D);
    // mod = 10
    buf.push(0b1000_0000 | (((r64 as u8) & 0b00000111) << 3) | (disp.reg as u8));
    encode_u32(disp.disp, buf);
}

fn encode_lea_r64_disp32_64(r64 : Reg64, disp : Disp32_Reg64, buf : &mut Vec<u8>) {
    let mut rexw = REX_W;
    if r64 >= Reg64::R8 {
        rexw |= 0b0000_0100;
    }
    if disp.reg >= Reg64::R8 {
        rexw |= 0b0000_0001;
    }
    buf.push(rexw);
    buf.push(0x8D);
    // mod = 10
    buf.push(0b1000_0000 | (((r64 as u8) & 0b00000111) << 3) | (disp.reg as u8));
    encode_u32(disp.disp, buf);
}

fn encode_pop_r(reg : Reg, buf : &mut Vec<u8>) {

    // This is tricky. Same encoding means "pop to 32bit reg" or "pop to 64bit
    // reg" depending on current mode (32bit vs. 64bit). Also, we need rex
    // prefix if 64bit regs are used, even thought the opcode column in the
    // manual doesn't show it.

    match reg {
        Reg::Reg8(_) => { panic!("encode_pop_r: Can't encode reg8"); },
        Reg::Reg16(reg16) => {
            buf.push(0x66);
            let (rex_b, reg_bits) = reg16_bits(reg16);
            if rex_b {
                buf.push(rex_pfx(false, false, false, true));
            }
            buf.push(0x58 + reg_bits);
        },
        Reg::Reg32(reg32) => {
            let (rex_b, reg_bits) = reg32_bits(reg32);
            if rex_b {
                buf.push(rex_pfx(false, false, false, true));
            }
            buf.push(0x58 + reg_bits);
        },
        Reg::Reg64(reg64) => {
            let (rex_b, reg_bits) = reg64_bits(reg64);
            if rex_b {
                buf.push(rex_pfx(false, false, false, true));
            }
            buf.push(0x58 + reg_bits);
        }
    }
}

fn encode_mov_r64_imm64(reg : Reg64, imm : u64, buf : &mut Vec<u8>) {
    let (rex_b, reg_bits) = reg64_bits(reg);
    buf.push(rex_pfx(true, false, false, rex_b));
    buf.push(0xB8 + reg_bits);
    encode_u64(imm, buf);
}

fn encode_movq_xmm_r64(xmm : XMM, reg : Reg64, buf : &mut Vec<u8>) {
    buf.push(0x66);
    let (rex_r, xmm_bits) = xmm_bits(xmm);
    let (rex_b, reg_bits) = reg64_bits(reg);
    buf.push(rex_pfx(true, rex_r, false, rex_b));
    buf.push(0x0F);
    buf.push(0x6E);
    // mod = 0b11
    buf.push(0b1100_0000 | (xmm_bits << 3) | reg_bits);
}

fn encode_sub_r64_ib(reg : Reg64, id : u8, buf : &mut Vec<u8>) {
    let (rex_b, reg_bits) = reg64_bits(reg);
    buf.push(rex_pfx(true, false, false, rex_b));
    buf.push(0x83);
    buf.push(0b1100_0000 | (5 << 3) | reg_bits);
    buf.push(id);
}

fn encode_add_r64_ib(reg : Reg64, id : u8, buf : &mut Vec<u8>) {
    let (rex_b, reg_bits) = reg64_bits(reg);
    buf.push(rex_pfx(true, false, false, rex_b));
    buf.push(0x83);
    buf.push(0b1100_0000 | reg_bits);
    buf.push(id);
}

fn encode_ret(buf : &mut Vec<u8>) {
    buf.push(0xC3);
}

fn encode_call_r64(reg : Reg64, buf : &mut Vec<u8>) {
    let (rex_b, reg_bits) = reg64_bits(reg);
    if rex_b {
        buf.push(rex_pfx(true, false, false, true));
    }
    buf.push(0xFF);
    buf.push(0b1100_0000 | (2 << 3) | reg_bits);
}

////////////////////////////////////////////////////////////////////////////////

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

    let mut buf : Vec<u8> = Vec::new();
    encode_adc_m64_r64(Reg64::RAX, Reg64::RDI, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_lea_r64_mem(Reg64::RAX, Reg::Reg64(Reg64::RDI), &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_lea_r64_disp8_32(Reg64::R11,  Disp8_Reg32 { reg : Reg32::EAX, disp : 15 }, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_lea_r64_disp8_64(Reg64::R11, Disp8_Reg64 { reg : Reg64::RAX, disp : 12 }, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_lea_r64_disp32_64(Reg64::R11,
                             Disp32_Reg64 { reg : Reg64::RAX, disp : 16777215 }, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_pop_r(Reg::Reg64(Reg64::R11), &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_pop_r(Reg::Reg64(Reg64::RAX), &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_pop_r(Reg::Reg64(Reg64::RAX), &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_pop_r(Reg::Reg16(Reg16::AX), &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_mov_r64_imm64(Reg64::RAX, 25.0f32 as u64, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_movq_xmm_r64(XMM::XMM0, Reg64::RAX, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_sub_r64_ib(Reg64::RSP, 8, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    encode_call_r64(Reg64::RAX, &mut buf);
    println!("{}", to_hex_string(&buf));
    buf.clear();

    let sqrt_ptr : u64 = unsafe {
        libc::dlsym(libc::RTLD_DEFAULT, CString::new("sqrt").unwrap().into_raw()) as u64
    };

    println!("ptr: 0x{:X}", sqrt_ptr);

    // A simple function that calls libc's sqrt
    buf.clear();
    encode_sub_r64_ib(Reg64::RSP, 8, &mut buf);
    encode_mov_r64_imm64(Reg64::RDI, 25.0f32 as u64, &mut buf);
    encode_movq_xmm_r64(XMM::XMM0, Reg64::RDI, &mut buf);
    // Call absolute address
    encode_mov_r64_imm64(Reg64::RSI, sqrt_ptr, &mut buf);
    encode_call_r64(Reg64::RSI, &mut buf);
    // Return value should be in correct register
    // move stack pointer back
    encode_add_r64_ib(Reg64::RSP, 8, &mut buf);
    encode_ret(&mut buf);
    println!("\n{}", to_hex_string(&buf));

    // let mut instrs = Vec::new();
    // for instr in instr_table::INSTR_STRS.iter() {
    //     let mnem = instr[0];
    //     let operands = instr[1];
    //     let encoding = instr[2];
    //     let opcodes = instr[3];
    //     let tags = instr[4];

    //     if let Some(opcode) = parse_opcode(opcodes) {
    //         if let Some(operands) = parse_operands(operands) {
    //             instrs.push(Instr_ {
    //                 mnem : mnem,
    //                 operands : operands,
    //                 // encoding : panic!(),
    //                 opcode : opcode
    //             });
    //         }
    //     }
    // }

    // println!("{} instructions parsed.", instrs.len());

    // for instr in instrs {
    //     println!("{:?}", instr);
    // }
}

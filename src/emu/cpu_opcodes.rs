use crate::emu::cpu_opcodes::Opcode::*;

#[derive(Copy, Clone)]
pub enum Opcode {
  NOP,
  BRK,
  JSR,
  RTI,
  RTS,

  JMP,
  JMPI,

  PHP,
  PLP,
  PHA,
  PLA,

  DEY,
  DEX,
  TAY,
  INY,
  INX,

  CLC,
  SEC,
  CLI,
  SEI,
  TYA,
  CLV,
  CLD,
  SED,

  TXA,
  TXS,
  TAX,
  TSX,

  UnknownOperation
}

impl Opcode {
  pub fn from_u8(value: u8) -> Opcode {
    match value {
      0xEA => NOP,
      0x00 => BRK,
      0x20 => JSR,
      0x40 => RTI,
      0x60 => RTS,
      0x4C => JMP,
      0x6C => JMPI,
      0x08 => PHP,
      0x28 => PLP,
      0x48 => PHA,
      0x68 => PLA,
      0x88 => DEY,
      0xCA => DEX,
      0xA8 => TAY,
      0xC8 => INY,
      0xE8 => INX,
      0x18 => CLC,
      0x38 => SEC,
      0x58 => CLI,
      0x78 => SEI,
      0x98 => TYA,
      0xB8 => CLV,
      0xD8 => CLD,
      0xF8 => SED,
      0x8A => TXA,
      0x9A => TXS,
      0xAA => TAX,
      0xBA => TSX,
      _ => UnknownOperation
    }
  }
}

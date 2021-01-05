use crate::emu::Bus::Bus;
use crate::emu::cpu_opcodes::Opcode;

pub enum InterruptType {
  IRQ,
  NMI,
  BRK_
}

// 6502 CPU
pub struct CPU {
  // Registers
  sp: u8, // Stack Pointer, Grows downward
  r_a: u8, // Accumulator
  r_x: u8, // Index
  r_y: u8, // Index
  pc: u16, // Program Counter
  // Cycle Counts
  cycles: u32,
  skip_cycles: u32,
  // Status flags
  f_c: bool,
  f_z: bool,
  f_i: bool,
  f_d: bool,
  f_v: bool,
  f_n: bool,
  bus: Bus
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      sp: 0x00,
      r_a: 0x00,
      r_x: 0x00,
      r_y: 0x00,
      pc: 0x0000,
      cycles: 0,
      skip_cycles: 0,
      f_c: false,
      f_z: false,
      f_i: false,
      f_d: false,
      f_v: false,
      f_n: false,
      bus: Bus::new(),
    }
  }

  pub fn reset(&mut self) {
    self.r_a = 0x00;
    self.r_x = 0x00;
    self.r_y = 0x00;

    self.f_c = false;
    self.f_z = false;
    self.f_i = false;
    self.f_d = false;
    self.f_v = false;
    self.f_n = false;

    self.cycles = 0;
    self.skip_cycles = 0;
  }

  pub fn step(&mut self) {
    self.cycles += 1;

    if self.skip_cycles > 0 {
      self.skip_cycles -= 1;
      return;
    }
    self.skip_cycles = 0;

    // Get opcode for next program counter target
    let op_byte = self.bus.read(self.pc);
    let opcode = Opcode::from_u8(op_byte);

    // Execute opcode
    self.executeOpcode(opcode);
    self.pc += 1;
  }

  pub fn executeOpcode(&mut self, opcode: Opcode) {

  }
}
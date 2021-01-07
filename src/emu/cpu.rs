use crate::emu::bus::Bus;
use crate::emu::cpu_opcodes::{Opcode, Instruction, AddressingMode};

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
  bus: Bus,

  location: u16,
  relative_location: u16
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
      location: 0x0000,
      relative_location: 0x0000
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
    let opcode = Instruction::from_u8(op_byte);

    // Execute opcode
    self.execute_opcode(&opcode);
    self.pc += 1;
  }

  pub fn execute_opcode(&mut self, instruction: &Instruction) {
    println!("EXECUTING OPCODE");
    if instruction.opcode == Opcode::ASL && instruction.addr_mode == AddressingMode::ZeroPage {
      println!("CORRECT OP");
    }
  }

  fn load_address_mode(&mut self, addr_mode: &AddressingMode) -> u8 {
    match addr_mode {
      AddressingMode::Implied => {
        return 0
      },
      AddressingMode::Immediate => {
        self.location = self.pc;
        self.pc += 1;
        return 0;
      },
      AddressingMode::ZeroPage => {
        let msb = self.bus.read(self.pc);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::ZeroPageX => {
        let msb = self.bus.read(self.pc + self.r_x as u16);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::ZeroPageY => {
        let msb = self.bus.read(self.pc + self.r_y as u16);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::Relative => {
        self.relative_location = self.bus.read(self.pc) as u16;
        self.pc += 1;
        if (self.relative_location & 0x80) != 0x0000 {
          self.relative_location |= 0xFF00;
        }
        return 0;
      },
      AddressingMode::Absolute => {
        let lo = self.bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.bus.read(self.pc) as u16;
        self.pc += 1;
        self.location = (hi << 8) | lo;
        return 0;
      },
      AddressingMode::AbsoluteX => {
        let lo = self.bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.bus.read(self.pc) as u16;
        self.pc += 1;

        self.location = (hi << 8) | lo;
        self.location += self.r_x as u16;

        if (self.location & 0xFF00) != (hi << 8) {
          return 1;
        } else {
          return 0;
        }
      },
      AddressingMode::AbsoluteY => {
        let lo = self.bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.bus.read(self.pc) as u16;
        self.pc += 1;

        self.location = (hi << 8) | lo;
        self.location += self.r_y as u16;

        if (self.location & 0xFF00) != (hi << 8) {
          return 1;
        } else {
          return 0;
        }
      },
      _ => { return 0; }
    }
  }
}
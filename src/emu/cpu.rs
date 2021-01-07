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
  pub sp: u8, // Stack Pointer, Grows downward
  pub r_a: u8, // Accumulator
  pub r_x: u8, // Index
  pub r_y: u8, // Index
  pub pc: u16, // Program Counter
  // Cycle Counts
  pub cycles: u32,
  pub skip_cycles: u8,
  // Status flags
  pub f_c: bool,
  pub f_z: bool,
  pub f_i: bool,
  pub f_d: bool,
  pub f_v: bool,
  pub f_n: bool,
  pub bus: Bus,

  pub location: u16,
  pub relative_location: u16,
  pub fetch_value: u8
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
      relative_location: 0x0000,
      fetch_value: 0x00
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

    // Get instruction from next program counter target
    let op_byte = self.bus.read(self.pc);
    let instruction = Instruction::from_u8(op_byte);
    self.pc += 1;

    // Execute instruction
    let wait_cycles = self.execute_instruction(&instruction);
    self.skip_cycles = wait_cycles;

    if self.skip_cycles > 0 {
      self.skip_cycles -= 1;
    }
  }

  pub fn execute_instruction(&mut self, instruction: &Instruction) -> u8 {
    let address_mode_cycles = self.load_address_mode(&instruction.addr_mode);

    let mut op_data = self.r_a;

    if instruction.addr_mode != AddressingMode::Implied {
      op_data = self.bus.read(self.location);
    }

    match instruction.opcode {
      Opcode::ADC => {
        let sum = self.r_a as u16 + op_data as u16 + self.f_c as u16;
        self.f_c = sum > 255;
        self.f_z = (sum & 0x00FF) == 0;
        self.f_v = (!((self.r_a as u16) ^ op_data as u16) & ((self.r_a as u16) ^ sum) & 0x0080) != 0;
        self.f_n = sum & 0x80 != 0;
        self.r_a = (sum & 0x00FF) as u8;
      },
      _ => {}
    }
    return instruction.cycles + address_mode_cycles;
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
      AddressingMode::Indirect => {
        let ptr_lo = self.bus.read(self.pc) as u16;
        self.pc += 1;
        let ptr_hi = self.bus.read(self.pc) as u16;
        self.pc += 1;

        let ptr = (ptr_hi << 8) | ptr_lo;

        // Page boundary hardware bug simulation
        if ptr_lo == 0x00FF {
          self.location = (self.bus.read(ptr & 0xFF00) as u16) | self.bus.read(ptr) as u16;
        } else {
          self.location = ((self.bus.read(ptr + 1) as u16) << 8) | self.bus.read(ptr) as u16;
        }

        return 0;
      },
      AddressingMode::IndirectX => {
        let address = self.bus.read(self.pc) as u16;
        self.pc += 1;

        let lo = self.bus.read((address + (self.r_x as u16)) & 0x00FF) as u16;
        let hi = self.bus.read((address + (self.r_x as u16) + 1) & 0x00FF) as u16;

        self.location = (hi << 8) | lo;
        return 0;
      },
      AddressingMode::IndirectY => {
        let address = self.bus.read(self.pc) as u16;
        self.pc += 1;

        let lo = self.bus.read(address & 0x00FF) as u16;
        let hi = self.bus.read((address + 0x0001) & 0x00FF) as u16;

        self.location = (hi << 8) | lo;
        self.location += self.r_y as u16;

        if (self.location & 0xFF00) != (hi << 8) {
          return 1;
        } else {
          return 0;
        }
      }
      _ => { return 0; }
    }
  }
}
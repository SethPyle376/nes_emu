use crate::emu::bus::Bus;
use crate::emu::cpu_opcodes::{Opcode, Instruction, AddressingMode};
use std::sync::{Arc, Mutex};

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
  pub r_status: u8,
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
  pub f_b: bool,
  pub f_u: bool,
  pub bus: Arc<Mutex<Bus>>,

  pub location: u16,
  pub relative_location: u16,
  pub fetch_value: u8
}

impl CPU {
  pub fn new(bus: Mutex<Bus>) -> CPU {
    CPU {
      sp: 0x00,
      r_a: 0x00,
      r_x: 0x00,
      r_y: 0x00,
      r_status: 0x00,
      pc: 0x0000,
      cycles: 0,
      skip_cycles: 0,
      f_c: false,
      f_z: false,
      f_i: false,
      f_d: false,
      f_v: false,
      f_n: false,
      f_b: false,
      f_u: false,
      bus: Arc::new(bus),
      location: 0x0000,
      relative_location: 0x0000,
      fetch_value: 0x00
    }
  }

  pub fn reset(&mut self) {
    self.r_a = 0x00;
    self.r_x = 0x00;
    self.r_y = 0x00;
    self.sp = 0xFD;
    self.r_status = 0x00 | ((self.f_u as u8) << 5);

    self.location = 0xFFFC;

    let lo = self.read(self.location);
    let hi = self.read(self.location + 1);

    self.pc = ((hi as u16) << 8) | (lo as u16);

    self.location = 0x0000;
    self.relative_location = 0x0000;

    self.f_c = false;
    self.f_z = false;
    self.f_i = false;
    self.f_d = false;
    self.f_v = false;
    self.f_n = false;

    self.cycles = 0;
    self.skip_cycles = 8;
  }

  pub fn interrupt(&mut self) {
    if self.f_i == true {
      return;
    }

    self.write(0x0100 + (self.sp as u16), (self.pc >> 8) as u8);
    self.sp -= 1;
    self.write(0x0100 + (self.sp as u16), (self.pc & 0x00FF) as u8);
    self.sp -= 1;

    self.f_b = false;
    self.f_u = true;
    self.f_i = true;

    self.write(0x0100 + (self.sp as u16), self.r_status);
    self.sp -= 1;

    self.location = 0xFFFE;

    let lo = self.read(self.location);
    let hi = self.read(self.location + 1);

    self.pc = ((hi as u16) << 8) | (lo as u16);

    self.skip_cycles = 7;
  }

  pub fn non_maskable_interrupt(&mut self) {
    self.write(0x0100 + (self.sp as u16), (self.pc >> 8) as u8);
    self.sp -= 1;
    self.write(0x0100 + (self.sp as u16), (self.pc & 0x00FF) as u8);
    self.sp -= 1;

    self.f_b = false;
    self.f_u = true;
    self.f_i = true;

    self.write(0x0100 + (self.sp as u16), self.r_status);
    self.sp -= 1;

    self.location = 0xFFFA;

    let lo = self.read(self.location);
    let hi = self.read(self.location + 1);

    self.pc = ((hi as u16) << 8) | (lo as u16);

    self.skip_cycles = 8;
  }

  pub fn step(&mut self) {
    self.cycles += 1;
    if self.skip_cycles > 0 {
      self.skip_cycles -= 1;
      return;
    }

    // Get instruction from next program counter target
    let op_byte = self.read(self.pc);
    let instruction = Instruction::from_u8(op_byte);
    self.pc += 1;

    // Execute instruction
    let wait_cycles = self.execute_instruction(&instruction);
    self.skip_cycles = wait_cycles;

    if self.skip_cycles > 0 {
      self.skip_cycles -= 1;
    }
  }
  
  pub fn read(&self, addr: u16) -> u8 {
    let data = self.bus.lock().unwrap();
    return data.read(addr);
  }
  
  pub fn write(&self, addr: u16, value: u8) {
    let mut data = self.bus.lock().unwrap();
    data.write(addr, value);
  }

  fn execute_instruction(&mut self, instruction: &Instruction) -> u8 {
    let address_mode_cycles = self.load_address_mode(&instruction.addr_mode);

    let mut op_data = self.r_a;

    if instruction.addr_mode != AddressingMode::Implied {
      op_data = self.read(self.location);
    }

    match instruction.opcode {
      Opcode::ADC => {
        let sum = self.r_a as u16 + op_data as u16 + self.f_c as u16;
        self.f_c = sum > 255;
        self.f_z = (sum & 0x00FF) == 0;
        self.f_v = (!((self.r_a as u16) ^ op_data as u16) & ((self.r_a as u16) ^ sum) & 0x0080) != 0;
        self.f_n = sum & 0x80 != 0;
        self.r_a = (sum & 0x00FF) as u8;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::SBC => {
        let inverted = (op_data as u16) ^ 0x00FF;
        let difference = self.r_a as u16 + inverted as u16 + self.f_c as u16 + 1;

        self.f_c = difference & 0xFF00 != 0;
        self.f_z = (difference & 0x00FF) == 0;
        self.f_v = ((difference ^ self.r_a as u16) & (difference ^ inverted) & 0x0080) != 0;
        self.f_n = difference & 0x0080 != 0;
        self.r_a = (difference & 0x00FF) as u8;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::AND => {
        self.r_a = self.r_a & op_data;
        self.f_z = self.r_a == 0x00;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::ASL => {
        let shifted = (op_data as u16) << 1;
        self.f_c = (shifted & 0xFF00) != 0;
        self.f_z = (shifted & 0x00FF) == 0x00;
        self.f_n = (shifted & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::BCC => {
        if self.f_c == false {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BCS => {
        if self.f_c == true {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BEQ => {
        if self.f_z == true {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BIT => {
        let bit = self.r_a & op_data;
        self.f_z = bit == 0x00;
        self.f_n = op_data & (1 << 7) != 0;
        self.f_v = op_data & (1 << 6) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BMI => {
        if self.f_n == true {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BNE => {
        if self.f_z == false {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BPL => {
        if self.f_n == false {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BRK => {
        self.f_i = true;
        self.write(0x0100 + self.sp as u16, ((self.pc >> 8) as u8) & 0x00FF);
        self.sp -= 1;
        self.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
        self.sp -= 1;

        self.f_b = true;
        self.write(0x0100 + self.sp as u16, self.r_status);
        self.sp -= 1;
        self.f_b = false;

        self.pc = self.read(0xFFFE) as u16 | self.read(0xFFFF) as u16;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BVC => {
        if self.f_v == false {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::BVS => {
        if self.f_v == true {
          self.location = self.pc + self.relative_location;
          self.pc = self.location;
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::CLC => {
        self.f_c = false;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::CLD => {
        self.f_d = false;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::CLI => {
        self.f_i = false;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::CLV => {
        self.f_v = false;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::CMP => {
        let difference = self.r_a as u16 - op_data as u16;
        self.f_c = self.r_a >= op_data;
        self.f_z = (difference & 0x00FF) == 0;
        self.f_n = (difference & 0x800) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::CPX => {
        let difference = self.r_x as u16 - op_data as u16;
        self.f_c = self.r_x >= op_data;
        self.f_z = (difference & 0x00FF) == 0x0000;
        self.f_n = (difference & 0x800) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::CPY => {
        let difference = self.r_y as u16 - op_data as u16;
        self.f_c = self.r_y >= op_data;
        self.f_z = (difference & 0x00FF) == 0x0000;
        self.f_n = (difference & 0x800) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::DEC => {
        let difference = op_data - 1;
        self.write(self.location, difference);
        self.f_z = (difference & 0x00FF) == 0;
        self.f_n = (difference & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::DEX => {
        self.r_x -= 1;
        self.f_z = self.r_x == 0;
        self.f_n = (self.r_x & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::DEY => {
        self.r_y -= 1;
        self.f_z = self.r_y == 0;
        self.f_n = (self.r_y & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::EOR => {
        self.r_a = self.r_a ^ op_data;
        self.f_z = self.r_a == 0;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::INC => {
        let value = op_data + 1;
        self.write(self.location, value & 0x00FF);
        self.f_z = (value & 0x00FF) == 0;
        self.f_n = (value & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::INX => {
        self.r_x += 1;
        self.f_z = self.r_x == 0;
        self.f_n = (self.r_x & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::INY => {
        self.r_y += 1;
        self.f_z = self.r_y == 0;
        self.f_n = (self.r_y & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::JMP => {
        self.pc = self.location;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::JSR => {
        self.pc -= 1;

        self.write(0x0100 + self.sp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.sp -= 1;
        self.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
        self.sp -= 1;

        self.pc = self.location;

        return address_mode_cycles + instruction.cycles;
      },
      Opcode::LDA => {
        self.r_a = op_data;
        self.f_z = self.r_a == 0x00;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::LDX => {
        self.r_x = op_data;
        self.f_z = self.r_x == 0x00;
        self.f_n = (self.r_x & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::LDY => {
        self.r_y = op_data;
        self.f_z = self.r_y == 0x00;
        self.f_n = (self.r_y & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::LSR => {
        self.f_c = (op_data & 0x0001) != 0;
        let shifted = op_data >> 1;
        self.f_z = shifted == 0;
        self.f_n = (shifted & 0x80) != 0;

        if instruction.addr_mode == AddressingMode::Implied {
          self.r_a = shifted;
        } else {
          self.write(self.location, shifted);
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::NOP => {
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::ORA => {
        self.r_a |= op_data;
        self.f_z = self.r_a == 0;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles + 1;
      },
      Opcode::PHA => {
        self.write(0x0100 + self.sp as u16, self.r_a);
        self.sp -= 1;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PHP => {
        let stored_status = self.r_status | (self.f_b as u8) << 4 | (self.f_u as u8) << 5;
        self.write(0x0100 + self.sp as u16, stored_status);
        self.f_b = false;
        self.f_u = false;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PLA => {
        self.sp += 1;
        self.r_a = self.read(self.sp as u16 + 0x0100);
        self.f_z = self.r_a == 0;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PLP => {
        self.sp += 1;
        self.r_status = self.read(0x100 + self.sp as u16);
        self.f_u = true;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::ROL => {
        let shifted = ((op_data as u16) << 1) | self.f_c as u16;
        self.f_c = (shifted & 0xFF00) != 0;
        self.f_z = (shifted & 0x00FF) == 0;
        self.f_n = (shifted & 0x80) != 0;

        if instruction.addr_mode == AddressingMode::Implied {
          self.r_a = (shifted & 0x00FF) as u8;
        } else {
          self.write(self.location, (shifted & 0x00FF) as u8);
        }
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::ROR => {
        let shifted = ((self.f_c as u16) << 7) | (op_data >> 1) as u16;
        self.f_c = (op_data & 0x01) != 0;
        self.f_z = (shifted & 0x00FF) == 0;
        self.f_n = (shifted & 0x80) != 0;

        if instruction.addr_mode == AddressingMode::Implied {
          self.r_a = (shifted & 0x00FF) as u8;
        } else {
          self.write(self.location, (shifted & 0x00FF) as u8);
        }

        return address_mode_cycles + instruction.cycles;
      },
      Opcode::RTI => {
        self.sp += 1;
        self.r_status = self.read(0x0100 + self.sp as u16);
        self.r_status &= ((!self.f_b) as u8) << 4;
        self.r_status &= ((!self.f_u) as u8) << 5;

        self.sp += 1;
        self.pc = self.read(self.sp as u16 + 0x100) as u16;
        self.sp += 1;
        self.pc |= (self.read(self.sp as u16 + 0x100) as u16) << 8;

        return address_mode_cycles + instruction.cycles;
      },
      Opcode::RTS => {
        self.sp += 1;
        self.pc = self.read(self.sp as u16 + 0x100) as u16;
        self.sp += 1;
        self.pc |= (self.read(self.sp as u16 + 0x100) as u16) << 8;

        self.pc += 1;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::SEC => {
        self.f_c = true;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::SED => {
        self.f_d = true;
        return address_mode_cycles + instruction.cycles;
      }
      Opcode::SEI => {
        self.f_i = true;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::STA => {
        self.write(self.location, self.r_a);
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::STX => {
        self.write(self.location, self.r_x);
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::STY => {
        self.write(self.location, self.r_y);
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::TAX => {
        self.r_x = self.r_a;
        self.f_z = self.r_x == 0x00;
        self.f_n = (self.r_x & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::TAY => {
        self.r_y = self.r_a;
        self.f_z = self.r_y == 0x00;
        self.f_n = (self.r_y & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::TSX => {
        self.r_x = self.sp;
        self.f_z = self.r_x == 0x00;
        self.f_n = (self.r_x & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::TXA => {
        self.r_a = self.r_x;
        self.f_z = self.r_a == 0x00;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::TXS => {
        self.sp = self.r_x;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::TYA => {
        self.r_a = self.r_y;
        self.f_z = self.r_a == 0x00;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::UnknownOperation => {
        return address_mode_cycles + instruction.cycles;
      }
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
        let msb = self.read(self.pc);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::ZeroPageX => {
        let msb = self.read(self.pc + self.r_x as u16);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::ZeroPageY => {
        let msb = self.read(self.pc + self.r_y as u16);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::Relative => {
        self.relative_location = self.read(self.pc) as u16;
        self.pc += 1;
        if (self.relative_location & 0x80) != 0x0000 {
          self.relative_location |= 0xFF00;
        }
        return 0;
      },
      AddressingMode::Absolute => {
        let lo = self.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.read(self.pc) as u16;
        self.pc += 1;
        self.location = (hi << 8) | lo;
        return 0;
      },
      AddressingMode::AbsoluteX => {
        let lo = self.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.read(self.pc) as u16;
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
        let lo = self.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.read(self.pc) as u16;
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
        let ptr_lo = self.read(self.pc) as u16;
        self.pc += 1;
        let ptr_hi = self.read(self.pc) as u16;
        self.pc += 1;

        let ptr = (ptr_hi << 8) | ptr_lo;

        // Page boundary hardware bug simulation
        if ptr_lo == 0x00FF {
          self.location = (self.read(ptr & 0xFF00) as u16) | self.read(ptr) as u16;
        } else {
          self.location = ((self.read(ptr + 1) as u16) << 8) | self.read(ptr) as u16;
        }

        return 0;
      },
      AddressingMode::IndirectX => {
        let address = self.read(self.pc) as u16;
        self.pc += 1;

        let lo = self.read((address + (self.r_x as u16)) & 0x00FF) as u16;
        let hi = self.read((address + (self.r_x as u16) + 1) & 0x00FF) as u16;

        self.location = (hi << 8) | lo;
        return 0;
      },
      AddressingMode::IndirectY => {
        let address = self.read(self.pc) as u16;
        self.pc += 1;

        let lo = self.read(address & 0x00FF) as u16;
        let hi = self.read((address + 0x0001) & 0x00FF) as u16;

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
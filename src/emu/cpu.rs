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

  pub location: u16,
  pub relative_location: u16,

  trace_file: Option<String>
}

impl CPU {
  pub fn new(trace_file: Option<String>) -> CPU {
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
      location: 0x0000,
      relative_location: 0x0000,
      trace_file
    }
  }

  pub fn reset(&mut self, bus: &mut Bus) {
    self.r_a = 0x00;
    self.r_x = 0x00;
    self.r_y = 0x00;
    self.sp = 0xFD;
    self.r_status = 0x00 | ((self.f_u as u8) << 5);

    self.location = 0xFFFC;

    let lo = bus.read(self.location);
    let hi = bus.read(self.location + 1);

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

  pub fn interrupt(&mut self, bus: &mut Bus) {
    if self.f_i == true {
      return;
    }

    bus.write(0x0100 + (self.sp as u16), (self.pc >> 8) as u8);
    self.sp -= 1;
    bus.write(0x0100 + (self.sp as u16), (self.pc & 0x00FF) as u8);
    self.sp -= 1;

    self.f_b = false;
    self.f_u = true;
    self.f_i = true;

    bus.write(0x0100 + (self.sp as u16), self.r_status);
    self.sp -= 1;

    self.location = 0xFFFE;

    let lo = bus.read(self.location);
    let hi = bus.read(self.location + 1);

    self.pc = ((hi as u16) << 8) | (lo as u16);

    self.skip_cycles = 7;
  }

  pub fn non_maskable_interrupt(&mut self, bus: &mut Bus) {
    bus.write(0x0100 + (self.sp as u16), (self.pc >> 8) as u8);
    self.sp -= 1;
    bus.write(0x0100 + (self.sp as u16), (self.pc & 0x00FF) as u8);
    self.sp -= 1;

    self.f_b = false;
    self.f_u = true;
    self.f_i = true;

    bus.write(0x0100 + (self.sp as u16), self.r_status);
    self.sp -= 1;

    self.location = 0xFFFA;

    let lo = bus.read(self.location);
    let hi = bus.read(self.location + 1);

    self.pc = ((hi as u16) << 8) | (lo as u16);

    self.skip_cycles = 8;
  }

  pub fn step(&mut self, bus: &mut Bus) {
    self.cycles += 1;
    if self.skip_cycles > 0 {
      self.skip_cycles -= 1;
      return;
    }

    // Get instruction from next program counter target
    let op_byte = bus.read(self.pc);
    let instruction = Instruction::from_u8(op_byte);

    if instruction.opcode == Opcode::UnknownOperation {
      panic!("UNKNOWN CPU OPERATION {}", op_byte);
    }

    if self.trace_file.is_some() {
      self.trace(bus);
    }

    self.pc += 1;

    // Execute instruction
    let wait_cycles = self.execute_instruction(&instruction, bus);
    self.skip_cycles = wait_cycles - 1;
  }
  
  pub fn read(&self, addr: u16, bus: &mut Bus) -> u8 {
    bus.read(addr)
  }
  
  pub fn write(&mut self, addr: u16, value: u8, bus: &mut Bus) {
    bus.write(addr, value);
  }

  fn execute_instruction(&mut self, instruction: &Instruction, bus: &mut Bus) -> u8 {
    let address_mode_cycles = self.load_address_mode(&instruction.addr_mode, bus);

    let mut op_data = self.r_a;

    if instruction.addr_mode != AddressingMode::Implied {
      op_data = bus.read(self.location);
    }

    match instruction.opcode {
      Opcode::ADC => {
        let sum = self.r_a as u16 + op_data as u16 + self.f_c as u16;
        self.f_c = sum > 255;
        self.f_z = (sum & 0x00FF) == 0;
        self.f_v = (!((self.r_a as u16) ^ op_data as u16) & ((self.r_a as u16) ^ sum) & 0x0080) != 0;
        self.f_n = sum & 0x80 != 0;
        self.r_a = (sum & 0x00FF) as u8;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::SBC => {
        let inverted = (op_data as u16) ^ 0x00FF;
        let difference = self.r_a as u16 + inverted as u16 + self.f_c as u16 + 1;

        self.f_c = difference & 0xFF00 != 0;
        self.f_z = (difference & 0x00FF) == 0;
        self.f_v = ((difference ^ self.r_a as u16) & (difference ^ inverted) & 0x0080) != 0;
        self.f_n = difference & 0x0080 != 0;
        self.r_a = (difference & 0x00FF) as u8;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::AND => {
        self.r_a = self.r_a & op_data;
        self.f_z = self.r_a == 0x00;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::ASL => {
        let shifted = (op_data as u16) << 1;
        self.f_c = (shifted & 0xFF00) != 0;
        self.f_z = (shifted & 0x00FF) == 0x00;
        self.f_n = (shifted & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
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
        bus.write(0x0100 + self.sp as u16, ((self.pc >> 8) as u8) & 0x00FF);
        self.sp -= 1;
        bus.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
        self.sp -= 1;

        self.f_b = true;
        bus.write(0x0100 + self.sp as u16, self.r_status);
        self.sp -= 1;
        self.f_b = false;

        self.pc = bus.read(0xFFFE) as u16 | bus.read(0xFFFF) as u16;
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
        return address_mode_cycles + instruction.cycles;
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
        bus.write(self.location, difference);
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
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::INC => {
        let value = op_data + 1;
        bus.write(self.location, value & 0x00FF);
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

        bus.write(0x0100 + self.sp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.sp -= 1;
        bus.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
        self.sp -= 1;

        self.pc = self.location;

        return address_mode_cycles + instruction.cycles;
      },
      Opcode::LDA => {
        self.r_a = op_data;
        self.f_z = self.r_a == 0x00;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::LDX => {
        self.r_x = op_data;
        self.f_z = self.r_x == 0x00;
        self.f_n = (self.r_x & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::LDY => {
        self.r_y = op_data;
        self.f_z = self.r_y == 0x00;
        self.f_n = (self.r_y & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::LSR => {
        self.f_c = (op_data & 0x0001) != 0;
        let shifted = op_data >> 1;
        self.f_z = shifted == 0;
        self.f_n = (shifted & 0x80) != 0;

        if instruction.addr_mode == AddressingMode::Implied {
          self.r_a = shifted;
        } else {
          bus.write(self.location, shifted);
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
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PHA => {
        bus.write(0x0100 + self.sp as u16, self.r_a);
        self.sp -= 1;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PHP => {
        let stored_status = self.r_status | (self.f_b as u8) << 4 | (self.f_u as u8) << 5;
        bus.write(0x0100 + self.sp as u16, stored_status);
        self.f_b = false;
        self.f_u = false;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PLA => {
        self.sp += 1;
        self.r_a = bus.read(self.sp as u16 + 0x0100);
        self.f_z = self.r_a == 0;
        self.f_n = (self.r_a & 0x80) != 0;
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::PLP => {
        self.sp += 1;
        self.r_status = bus.read(0x100 + self.sp as u16);
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
          bus.write(self.location, (shifted & 0x00FF) as u8);
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
          bus.write(self.location, (shifted & 0x00FF) as u8);
        }

        return address_mode_cycles + instruction.cycles;
      },
      Opcode::RTI => {
        self.sp += 1;
        self.r_status = bus.read(0x0100 + self.sp as u16);
        self.r_status &= ((!self.f_b) as u8) << 4;
        self.r_status &= ((!self.f_u) as u8) << 5;

        self.sp += 1;
        self.pc = bus.read(self.sp as u16 + 0x100) as u16;
        self.sp += 1;
        self.pc |= (bus.read(self.sp as u16 + 0x100) as u16) << 8;

        return address_mode_cycles + instruction.cycles;
      },
      Opcode::RTS => {
        self.sp += 1;
        self.pc = bus.read(self.sp as u16 + 0x100) as u16;
        self.sp += 1;
        self.pc |= (bus.read(self.sp as u16 + 0x100) as u16) << 8;

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
        bus.write(self.location, self.r_a);
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::STX => {
        bus.write(self.location, self.r_x);
        return address_mode_cycles + instruction.cycles;
      },
      Opcode::STY => {
        bus.write(self.location, self.r_y);
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

  fn load_address_mode(&mut self, addr_mode: &AddressingMode, bus: &mut Bus) -> u8 {
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
        let msb = bus.read(self.pc);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::ZeroPageX => {
        let msb = bus.read(self.pc + self.r_x as u16);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::ZeroPageY => {
        let msb = bus.read(self.pc + self.r_y as u16);
        self.pc += 1;
        self.location = msb as u16 & 0x00FF;
        return 0;
      },
      AddressingMode::Relative => {
        self.relative_location = bus.read(self.pc) as u16;
        self.pc += 1;
        if (self.relative_location & 0x80) != 0x0000 {
          self.relative_location |= 0xFF00;
        }
        return 0;
      },
      AddressingMode::Absolute => {
        let lo = bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc) as u16;
        self.pc += 1;
        self.location = (hi << 8) | lo;
        return 0;
      },
      AddressingMode::AbsoluteX => {
        let lo = bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc) as u16;
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
        let lo = bus.read(self.pc) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc) as u16;
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
        let ptr_lo = bus.read(self.pc) as u16;
        self.pc += 1;
        let ptr_hi = bus.read(self.pc) as u16;
        self.pc += 1;

        let ptr = (ptr_hi << 8) | ptr_lo;

        // Page boundary hardware bug simulation
        if ptr_lo == 0x00FF {
          self.location = (bus.read(ptr & 0xFF00) as u16) | bus.read(ptr) as u16;
        } else {
          self.location = ((bus.read(ptr + 1) as u16) << 8) | bus.read(ptr) as u16;
        }

        return 0;
      },
      AddressingMode::IndirectX => {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let lo = bus.read((address + (self.r_x as u16)) & 0x00FF) as u16;
        let hi = bus.read((address + (self.r_x as u16) + 1) & 0x00FF) as u16;

        self.location = (hi << 8) | lo;
        return 0;
      },
      AddressingMode::IndirectY => {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let lo = bus.read(address & 0x00FF) as u16;
        let hi = bus.read((address + 0x0001) & 0x00FF) as u16;

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

  fn trace(&self, bus: &mut Bus) {
    let opcode = bus.read(self.pc);
    let instruction = Instruction::from_u8(opcode);

    let mut instruction_bytes = vec![opcode];

    match instruction.addr_mode {
      AddressingMode::Immediate | AddressingMode::ZeroPage | AddressingMode::ZeroPageX | AddressingMode::ZeroPageY
        | AddressingMode::IndirectX | AddressingMode::IndirectY | AddressingMode::Relative => {
        instruction_bytes.push(bus.read(self.pc + 1));
      }
      AddressingMode::Absolute | AddressingMode::AbsoluteX | AddressingMode::AbsoluteY | AddressingMode::Indirect => {
        instruction_bytes.push(bus.read(self.pc + 1));
        instruction_bytes.push(bus.read(self.pc + 2));
      }
      _ => {}
    };

    let byte_str = instruction_bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join(" ");
    let instruction_string = format!("{:04x}  {:8} {: >4}", self.pc, byte_str, instruction.opcode);

    let trace_string = format!("{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}", 
      instruction_string, self.r_a, self.r_x, self.r_y, self.r_status, self.sp).to_ascii_uppercase();

    println!("{}", trace_string);
  }
}
use super::cartridge::Mirroring;

const VRAM_ADD_INCREMENT_BIT : u8 = 0b100;

pub struct PPU {
  pub chr_rom: Vec<u8>,
  pub palette_table: [u8; 32],
  pub vram: [u8; 2048],
  pub oam_data: [u8; 256],
  pub mirroring: Mirroring,

  byte_buffer: u8,
  
  mem_addr_reg: MemoryAddressRegister,
  control_reg: u8,

  cycles: usize
}

impl PPU {
  pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> PPU {
    PPU {
      chr_rom,
      mirroring,
      vram: [0; 2048],
      oam_data: [0; 256],
      palette_table: [0; 32],
      mem_addr_reg: MemoryAddressRegister::default(),
      control_reg: 0,
      byte_buffer: 0,
      cycles: 0
    }
  }

  pub fn read(&mut self) -> u8 {
    let addr = self.mem_addr_reg.value;
    self.increment_addr();

    match addr {
      0 ..= 0x1FFF => {
        let data = self.byte_buffer;
        self.byte_buffer = self.chr_rom[addr as usize];
        data
      },
      0x2000 ..= 0x2FFF => self.vram[self.mirror_addr(addr) as usize],
      0x3F00 ..= 0x3FFF => self.palette_table[addr as usize & 0xFF],
      _ => panic!("BAD MEMORY SPACE ACCESS")
    }
  }

  pub fn write(&mut self, addr: u16, data: u8) {
    match addr {
      0x2016 => {
        self.mem_addr_reg.write(data);
      }
      _ => panic!("BAD MEMORY SPACE WRITE")
    }
  }

  fn mirror_addr(&self, addr: u16) -> u16 {
    let vram_index = addr & 0xEFF;
    let name_table_index = vram_index / 0x400;

    if name_table_index == 0 || (self.mirroring == Mirroring::Vertical && name_table_index == 1) {
      return vram_index;
    } else if self.mirroring == Mirroring::Horizontal {
      return vram_index - (0x400 + (name_table_index - 1) * 0x400);
    } else if self.mirroring == Mirroring::Vertical {
      return vram_index - 0x800;
    } else {
      panic!("FOUR SCREEN VRAM MIRRORING NOT SUPPORTED");
    }
  }

  fn increment_addr(&mut self) {
    let value = if self.control_reg & VRAM_ADD_INCREMENT_BIT != 0 { 1 } else { 32 };
    self.mem_addr_reg.increment(value);
  }
}

struct MemoryAddressRegister {
  value: u16,
  top_byte_set: bool
}

impl Default for MemoryAddressRegister {
  fn default() -> Self {
      MemoryAddressRegister { value: 0, top_byte_set: false }
  }
}

impl MemoryAddressRegister {
  pub fn write(&mut self, byte: u8) {
    if self.top_byte_set {
      self.value = self.value | byte as u16;
    } else {
      self.value = self.value | ((byte as u16) << 8);
    }

    self.value |= 0x3FFF;

    self.top_byte_set = !self.top_byte_set;
  }

  pub fn increment(&mut self, value: u8) {
    self.value += value as u16;
    self.value |= 0x3FFF;
  }
}
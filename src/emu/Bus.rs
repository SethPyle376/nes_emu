use crate::emu::ppu::PPU;
use crate::emu::cartridge::Cartridge;

// RAM Addresses
const RAM_BEGIN: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

// PPU Register Addresses
const PPU_REGISTER_BEGIN: u16 = 0x2000;
const PPU_REGISTER_END: u16 = 0x3FFF;

pub struct Bus {
  pub ram: Vec<u8>,
  pub ppu: PPU,
  pub cartridge: Cartridge
}

impl Bus {
  pub fn new(cartridge: Cartridge) -> Bus {
    let mut bus = Bus {
      ram: Vec::with_capacity(0x800),
      ppu: PPU::new(),
      cartridge
    };
    bus.ram.resize(0x800, 0x00);
    return bus;
  }

  pub fn read(&self, addr: u16) -> u8 {
    match addr {
      // Main RAM read
      RAM_BEGIN ..= RAM_END => {
        return self.ram[usize::from(addr & 0x7FF)];
      }
      PPU_REGISTER_BEGIN ..= PPU_REGISTER_END => {
        todo!("PPU REGISTER READS NOT YET SUPPORTED");
      }
      _ => {
        println!("IGNORING MEMORY READ AT ADDRESS {}", addr);
        return 0;
      }
    }
  }

  pub fn write(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      self.ram[usize::from(addr & 0x7FF)] = value;
    }
  }
}
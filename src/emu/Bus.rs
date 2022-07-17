use crate::emu::ppu::PPU;
use crate::emu::cartridge::Cartridge;

// RAM Addresses
const RAM_BEGIN: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

// PPU Register Addresses
const PPU_REGISTER_BEGIN: u16 = 0x2000;
const PPU_REGISTER_END: u16 = 0x3FFF;

const PRG_ROM_BEGIN: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

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
      PRG_ROM_BEGIN ..= PRG_ROM_END => {
        let mut rom_location = addr - 0x8000;
        
        if self.cartridge.prg_rom.len() == 0x4000 {
          rom_location = rom_location % 0x4000;
        }

        return self.cartridge.prg_rom[rom_location as usize];
      }
      _ => {
        println!("IGNORING MEMORY READ AT ADDRESS {}", addr);
        return 0;
      }
    }
  }

  pub fn write(&mut self, addr: u16, value: u8) {
    match addr {
      RAM_BEGIN ..= RAM_END => {
        self.ram[usize::from(addr & 0x7FF)] = value;
      }
      PPU_REGISTER_BEGIN ..= PPU_REGISTER_END => {
        todo!("PPU REGISTER READS NOT YET SUPPORTED");
      }
      PRG_ROM_BEGIN ..= PRG_ROM_END => {
        panic!("WRITE TO PRG ROM ATTEMPTED");
      }
      _ => {
        println!("IGNORING MEMORY WRITE AT ADDRESS {}", addr);
      }
    }
  }
}
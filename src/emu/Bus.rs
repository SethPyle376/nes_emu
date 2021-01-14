use std::sync::{Mutex, Arc};
use crate::emu::ppu::PPU;

pub struct Bus {
  pub ram: Vec<u8>,
  pub ppu: Arc<Mutex<PPU>>
}

impl Bus {
  pub fn new(ppu: &Arc<Mutex<PPU>>) -> Bus {
    let mut bus = Bus {
      ram: Vec::with_capacity(0x800),
      ppu: Arc::clone(ppu)
    };
    bus.ram.resize(0x800, 0x00);
    return bus;
  }

  pub fn read(&self, addr: u16) -> u8 {
    // Main RAM read
    if addr < 0x2000 {
      // Strip off anything greater than RAM capacity (0x7FF)
      // Memory 0x0 to 0x7FF is mirrored 3 more times up to 0x2000 where PPU registers start
      return self.ram[usize::from(addr & 0x7FF)];
    } else if addr < 0x4000 {
      let ppu_addr = (addr & 0x0007) as u8;
      let mut lock = self.ppu.lock().unwrap();
      return lock.cpu_read(ppu_addr) as u8;
    } else {
      return 0;
    }
  }

  pub fn write(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      self.ram[usize::from(addr & 0x7FF)] = value;
    }
  }
}
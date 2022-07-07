use crate::emu::cpu::CPU;
use crate::emu::ppu::PPU;

pub struct NES {
  pub cpu: CPU,
  pub ppu: PPU
}

impl NES {
  pub fn new() -> NES {
    NES {
      cpu: CPU::new(),
      ppu: PPU::new()
    }
  }
}
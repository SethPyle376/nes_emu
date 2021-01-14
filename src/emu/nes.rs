use std::sync::{Arc, Mutex};
use crate::emu::cpu::CPU;
use crate::emu::ppu::PPU;
use crate::emu::bus::Bus;

pub struct NES {
  pub cpu: Arc<Mutex<CPU>>,
  pub ppu: Arc<Mutex<PPU>>,
  pub bus: Arc<Mutex<Bus>>
}

impl NES {
  pub fn new() -> NES {
    let ppu = Arc::new(Mutex::new(PPU::new()));
    let bus = Arc::new(Mutex::new(Bus::new(&Arc::clone(&ppu))));
    NES {
      cpu: Arc::new(Mutex::new(CPU::new(&Arc::clone(&bus)))),
      ppu,
      bus
    }
  }
}
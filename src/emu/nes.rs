use crate::emu::cpu::CPU;

pub struct NES {
  pub cpu: CPU
}

impl NES {
  pub fn new() -> NES {
    NES {
      cpu: CPU::new(None)
    }
  }
}
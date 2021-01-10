#![allow(dead_code)]
mod emu;
use emu::cpu::CPU;
use emu::debug_interface::DebugInterface;
use std::sync::{Mutex, Arc};

fn main() {
  let bus = Mutex::new(emu::bus::Bus::new());
  let mut cpu = Arc::new(Mutex::new(CPU::new(bus)));

  let interface = DebugInterface::new(&cpu);
  interface.test();

  cpu.lock().unwrap().pc = 0xFF;
  cpu.lock().unwrap().write(0x0000, 0x69);

  interface.test();
}

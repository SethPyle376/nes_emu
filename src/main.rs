#![allow(dead_code)]
mod emu;
use emu::cpu::CPU;
use emu::debug_interface::DebugInterface;
use std::sync::{Mutex, Arc};

fn main() {
  let bus = Mutex::new(emu::bus::Bus::new());
  let cpu = Arc::new(Mutex::new(CPU::new(bus)));

  let mut interface = DebugInterface::new(&cpu, 60);

  cpu.lock().unwrap().pc = 0xFF;
  cpu.lock().unwrap().write(0x0000, 0x69);
  cpu.lock().unwrap().write(0x0001, 0x70);
  cpu.lock().unwrap().write(0x0002, 0x71);
  cpu.lock().unwrap().write(0x00FF, 0x42);

  let mut cycles = 0;

  while interface.should_quit != true {
    interface.draw();
    cpu.lock().unwrap().bus.lock().unwrap().ram[0x0003] += 1;
    cycles += 1;
  }
}

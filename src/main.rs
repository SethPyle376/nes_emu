#![allow(dead_code)]
mod emu;
use emu::cpu::CPU;
use std::sync::Mutex;

fn main() {
  let bus = Mutex::new(emu::bus::Bus::new());
  let mut cpu = CPU::new(bus);
  let mut cycle_count = 0;
  let cycle_max = 100;
  while cycle_count < cycle_max {
    cpu.step();
    cycle_count += 1;
  }
}

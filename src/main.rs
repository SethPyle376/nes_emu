#![allow(dead_code)]
mod emu;
use emu::cpu::CPU;
use emu::debug_interface::DebugInterface;
use std::sync::{Mutex, Arc};
use clap::Clap;
use std::time::Instant;

#[derive(Clap)]
struct Opts {
  #[clap(long, parse(try_from_str), default_value = "false")]
  pub debug_interface: bool,
}

fn main() {
  let opts = Opts::parse();
  let bus = Arc::new(Mutex::new(emu::bus::Bus::new()));
  let cpu = Arc::new(Mutex::new(CPU::new(&bus)));

  let mut interface = DebugInterface::new(&cpu, 10);

  let mut cycles = 0;

  {
    let mut lock = bus.lock().unwrap();
    lock.ram[0x0000] = 0x69;
    lock.ram[0x0001] = 0x24;
    lock.ram[0x0002] = 0x69;
    lock.ram[0x0003] = 0x32;
    lock.ram[0x0004] = 0x4C;
    lock.ram[0x0005] = 0x00;
    lock.ram[0x0006] = 0x00;
  }

  let mut duration_total : u128 = 0;

  while cycles < 20_560_000 {
    let first = Instant::now();
    if opts.debug_interface {
      interface.draw();
    }
    cpu.lock().unwrap().step();
    cycles += 1;
    let second = Instant::now();

    let duration = second.duration_since(first);
    duration_total += duration.as_nanos()
  }
  interface.cleanup();

  println!("{:?} average FPS", 1_000_000_000 / (duration_total / 20560000));
}

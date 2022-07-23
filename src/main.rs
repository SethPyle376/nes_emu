#![allow(dead_code)]
mod emu;
mod graphics;

use clap::Clap;
use std::time::Instant;
use crate::emu::cpu::CPU;
use crate::emu::bus::Bus;
use crate::emu::cartridge::Cartridge;

#[derive(Clap)]
struct Opts {
  #[clap(short, long)]
  pub rom_path: String
}

fn main() {
  let opts = Opts::parse();
  let mut cpu = CPU::new(Some("blah".to_string()));

  let cycle_count = 20_000;
  let mut cycles = 0;

  let mut bus = Bus::new(Cartridge::load(&opts.rom_path.as_str()).unwrap());
  cpu.reset(&mut bus);

  let start = Instant::now();
  while cycles < cycle_count {
    cpu.step(&mut bus);
    cycles += 1;
  }
  let end = Instant::now();

  let duration = end - start;

  let cycles_per_second = cycle_count as f64 / duration.as_secs_f64();

  println!("{} CYCLES PER SECOND", cycles_per_second);
}

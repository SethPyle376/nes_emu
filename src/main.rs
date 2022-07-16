#![allow(dead_code)]
mod emu;

use clap::Clap;
use std::time::Instant;
use crate::emu::nes::NES;
use crate::emu::bus::Bus;
use crate::emu::cartridge::Cartridge;

#[derive(Clap)]
struct Opts {
  #[clap(short, long)]
  pub rom_path: String
}

fn main() {
  let opts = Opts::parse();
  let mut nes = NES::new();

  let cycle_count = 100_000_000;
  let mut cycles = 0;

  let mut bus = Bus::new(Cartridge::load(&opts.rom_path.as_str()).unwrap());
  bus.ram[0x0000] = 0x69;
  bus.ram[0x0001] = 0x24;
  bus.ram[0x0002] = 0x69;
  bus.ram[0x0003] = 0x32;
  bus.ram[0x0004] = 0x4C;
  bus.ram[0x0005] = 0x00;
  bus.ram[0x0006] = 0x00;

  let start = Instant::now();
  while cycles < cycle_count {
    nes.cpu.step(&mut bus);
    cycles += 1;
  }
  let end = Instant::now();

  let duration = end - start;

  let cycles_per_second = cycle_count as f64 / duration.as_secs_f64();

  println!("{} CYCLES PER SECOND", cycles_per_second);
}

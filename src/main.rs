#![allow(dead_code)]
mod emu;
use emu::cpu::CPU;
use emu::debug_interface::DebugInterface;
use std::sync::{Mutex, Arc};
use clap::Clap;

// fn str_to_bool(s: &str) -> Result<bool, &'static str> {
//   match s {
//     "true" => Ok(true),
//     "false" => Ok(false),
//     _ => Err("expected true or false")
//   }
// }

#[derive(Clap)]
struct Opts {
  #[clap(long, parse(try_from_str), default_value = "false")]
  pub debug_interface: bool,
}

fn main() {
  let opts = Opts::parse();
  let bus = Arc::new(Mutex::new(emu::bus::Bus::new()));
  let cpu = Arc::new(Mutex::new(CPU::new(&bus)));

  let mut interface = DebugInterface::new(&cpu, 30);

  let mut cycles = 0;

  while cycles < 256 {
    if opts.debug_interface {
      interface.draw();
    }
    cycles += 1;
  }
  interface.cleanup();
}

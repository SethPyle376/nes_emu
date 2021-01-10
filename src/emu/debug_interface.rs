use crate::emu::cpu::CPU;
use std::sync::{Arc, Mutex};
use crate::emu::bus::Bus;

use std::str;

pub struct DebugInterface {
  cpu: Arc<Mutex<CPU>>,
  bus: Arc<Mutex<Bus>>
}

impl DebugInterface {
  pub fn new(cpu: &Arc<Mutex<CPU>>) -> DebugInterface{
    DebugInterface {
      cpu: cpu.clone(),
      bus: Arc::clone(&cpu.lock().unwrap().bus)
    }
  }

  pub fn test(&self) {
    println!("{}", self.cpu.lock().unwrap().pc);

    let lock = self.bus.lock().unwrap();
    let ram = lock.ram.to_vec();

    let ram_hex = hex::encode(&ram);

    let formatted = ram_hex.as_bytes().chunks(2).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();

    println!("{:?}", &formatted);
  }
}
#![allow(dead_code)]
extern crate nes_emu;

mod cpu_tests {
  use nes_emu::emu;
  use std::sync::{Mutex, Arc};

  fn run_cpu_cycles(cpu: &mut emu::cpu::CPU, cycles: u32) {
    for _x in 0..cycles {
      cpu.step()
    }
  }

  #[test]
  fn adc_imm_test() {
    let bus = Arc::new(Mutex::new(emu::bus::Bus::new()));
    let mut cpu = emu::cpu::CPU::new(&bus);
    cpu.write(0x0000, 0x69);
    cpu.write(0x0001, 0x24);
    cpu.write(0x0002, 0x69);
    cpu.write(0x0003, 0x32);
    run_cpu_cycles(&mut cpu, 4);
    assert_eq!(cpu.r_a, 0x56);
  }

  #[test]
  fn adc_abs_test() {
    let bus = Arc::new(Mutex::new(emu::bus::Bus::new()));
    let mut cpu = emu::cpu::CPU::new(&bus);
    cpu.write(0x0000, 0x6D);
    cpu.write(0x0001, 0x00);
    cpu.write(0x0002, 0x04);
    cpu.write(0x0003, 0x6D);
    cpu.write(0x0004, 0x00);
    cpu.write(0x0005, 0x05);

    cpu.write(0x400, 0x69);
    cpu.write(0x500, 0x43);

    run_cpu_cycles(&mut cpu, 8);

    assert_eq!(cpu.r_a, 0xAC);
  }

  #[test]
  fn sbc_imm_test() {
    let bus = Arc::new(Mutex::new(emu::bus::Bus::new()));
    let mut cpu = emu::cpu::CPU::new(&bus);
    cpu.r_a = 0x69;
    cpu.write(0x0000, 0xE9);
    cpu.write(0x0001, 0x42);

    run_cpu_cycles(&mut cpu, 2);

    assert_eq!(cpu.r_a, 0x27);
  }
}

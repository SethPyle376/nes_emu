#![allow(dead_code)]
extern crate nes_emu;

mod cpu_tests {
  use nes_emu::emu;

  fn run_cpu_cycles(cpu: &mut emu::cpu::CPU, cycles: u32, bus: &mut emu::bus::Bus) {
    for _x in 0..cycles {
      cpu.step(bus)
    }
  }

  #[test]
  fn adc_imm_test() {
    let mut cpu = emu::cpu::CPU::new();
    let mut bus = emu::bus::Bus::new();
    bus.write(0x0000, 0x69);
    bus.write(0x0001, 0x24);
    bus.write(0x0002, 0x69);
    bus.write(0x0003, 0x32);
    run_cpu_cycles(&mut cpu, 4, &mut bus);
    assert_eq!(cpu.r_a, 0x56);
  }

  #[test]
  fn adc_abs_test() {
    let mut cpu = emu::cpu::CPU::new();
    let mut bus = emu::bus::Bus::new();
    bus.write(0x0000, 0x6D);
    bus.write(0x0001, 0x00);
    bus.write(0x0002, 0x04);
    bus.write(0x0003, 0x6D);
    bus.write(0x0004, 0x00);
    bus.write(0x0005, 0x05);

    bus.write(0x400, 0x69);
    bus.write(0x500, 0x43);

    run_cpu_cycles(&mut cpu, 8, &mut bus);

    assert_eq!(cpu.r_a, 0xAC);
  }

  #[test]
  fn sbc_imm_test() {
    let mut cpu = emu::cpu::CPU::new();
    let mut bus = emu::bus::Bus::new();
    cpu.r_a = 0x69;
    bus.write(0x0000, 0xE9);
    bus.write(0x0001, 0x42);

    run_cpu_cycles(&mut cpu, 2, &mut bus);

    assert_eq!(cpu.r_a, 0x27);
  }
}

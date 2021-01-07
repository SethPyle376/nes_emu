mod emu;
use emu::cpu::CPU;

fn main() {
  let mut cpu = CPU::new();
  let mut cycle_count = 0;
  let cycle_max = 100;
  while cycle_count < cycle_max {
    cpu.step();
    cycle_count += 1;
  }
}

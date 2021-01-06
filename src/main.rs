mod emu;
use emu::cpu::CPU;

fn main() {
  let mut cpu = CPU::new();
  cpu.step();
  cpu.reset();
}

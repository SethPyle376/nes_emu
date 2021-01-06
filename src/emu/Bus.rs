pub struct Bus {
  ram: Vec<u8>,
}

impl Bus {
  pub fn new() -> Bus {
    let mut bus = Bus {
      ram: Vec::with_capacity(0x800)
    };
    bus.ram.resize(0x800, 0x00);
    bus.ram[0] = 0x06;
    return bus;
  }

  pub fn read(&self, addr: u16) -> u8 {
    // Main RAM read
    if addr < 0x2000 {
      // Strip off anything greater than RAM capacity (0x7FF)
      // Memory 0x0 to 0x7FF is mirrored 3 more times up to 0x2000 where PPU registers start
      return self.ram[usize::from(addr & 0x7FF)];
    } else if addr < 0x4020 {
      // PPU
      return 0;
    } else {
      return 0;
    }
  }
}
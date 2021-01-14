pub struct PPU {
  pub control: u8,
  pub mask: u8,
  pub status: u8,
  pub oam_addr: u8,
  pub oam_data: u8,
  pub scroll: u8,
  pub addr: u8,
  pub data: u8,
  pub oam_dma: u8
}

impl PPU {
  pub fn new() -> PPU {
    PPU {
      control: 0x00,
      mask: 0x00,
      status: 0x00,
      oam_addr: 0x00,
      oam_data: 0x00,
      scroll: 0x00,
      addr: 0x00,
      data: 0x00,
      oam_dma: 0x00
    }
  }
  pub fn cpu_read(&mut self, addr: u8) -> u8 { return 0x00; }
  pub fn cpu_write(&mut self, addr: u8, data: u8) {}
}
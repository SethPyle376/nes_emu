const WIDTH: usize = 256;
const HEIGHT: usize = 240;

pub struct Frame {
  pub data: Vec<u8>
}

impl Default for Frame {
  fn default() -> Self {
      Frame { data: vec![0; WIDTH * HEIGHT * 3] }
  }
}

impl Frame {
  pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
    let first = (y * 3 * WIDTH) + (x * 3);

    self.data[first] = color.0;
    self.data[first + 1] = color.1;
    self.data[first + 2] = color.2;
  }
}
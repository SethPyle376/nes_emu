#[derive(Copy, Clone)]
pub enum Opcode {
  TestOp,
}

impl Opcode {
  pub fn from_u8(value: u8) -> Opcode {
    match value {
      0xFF => Opcode::TestOp,
      _ => panic!("Unknown opcode: {}", value)
    }
  }
}

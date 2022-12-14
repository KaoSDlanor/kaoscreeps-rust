use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdGenerator {
  pub next_id: usize,
}

impl IdGenerator {
  pub fn new() -> Self {
    Self {
      next_id : 0,
    }
  }

  pub fn generate(&mut self) -> usize {
    let out = self.next_id;
    self.next_id += 1;
    out
  }
}

impl Default for IdGenerator {
  fn default() -> Self {
    Self::new()
  }
}
use serde::{Serialize, Deserialize};

pub const MEM_VERSION: (u8,u8,u8,u8) = (0,0,0,0);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory {
  pub version: (u8,u8,u8,u8),
  pub hive : crate::hive::Hive,
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      version: MEM_VERSION,

      hive : crate::hive::Hive::new(screeps::game::rooms().values().next().unwrap()),
    }
  }

  pub fn run(&mut self) {
    self.hive.run()
  }
}

pub fn load() -> Memory {
  let serialized = String::from(screeps::raw_memory::RawMemory::get());
  let memory: Memory = serde_json::from_str(&serialized).unwrap_or_else(|_| Memory::new());
  if memory.version == MEM_VERSION {
    memory
  } else {
    Memory::new()
  }
}

pub fn save(memory: &Memory) {
  let serialized = serde_json::to_string(memory).unwrap();
  screeps::raw_memory::RawMemory::set(&serialized.into());
}
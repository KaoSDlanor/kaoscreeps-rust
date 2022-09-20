use std::sync::Mutex;
use wasm_bindgen::prelude::*;

pub mod console;
pub mod constants;
pub mod helpers;
pub mod hive;

#[wasm_bindgen]
pub fn game_loop() {
  static MEM: Mutex<Option<helpers::memory::Memory>> = Mutex::new(None);
  let mut mem_ref = MEM.lock().unwrap();

  let mut mem = mem_ref.clone().unwrap_or_else(|| helpers::memory::Memory::new());

  mem.hive.run();

  console::info(format!("{:?}",&mem));
  helpers::memory::save(&mem);

  *mem_ref = Some(mem);
}
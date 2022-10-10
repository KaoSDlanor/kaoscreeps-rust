extern crate console_error_panic_hook;

// use std::sync::Mutex;
use wasm_bindgen::prelude::*;

pub mod console;
pub mod constants;
pub mod hive;
pub mod memory;

#[wasm_bindgen]
pub fn setup() {
  console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn game_loop() {
  // static MEM: Mutex<Option<memory::Memory>> = Mutex::new(None);
  // let mut mem_ref = MEM.lock().unwrap();

  // let mut mem = mem_ref.to_owned().unwrap_or_else(|| memory::Memory::new());
  let mut mem = memory::load();

  mem.run();

  memory::save(&mem);

  // *mem_ref = Some(mem);
}
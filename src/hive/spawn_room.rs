use screeps::{HasId, StructureObject, ObjectId, RawObjectId, StructureSpawn, StructureExtension, Room, find, RoomName, game};
use serde::{Serialize, Deserialize};

use std::{cell::RefCell};

#[derive(Debug, Clone)]
struct CachedSpawnList(u32,Option<RefCell<Vec<RawObjectId>>>);

impl Default for CachedSpawnList {
  fn default() -> Self {
    Self(game::time(), None)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpawnRoom {
  pub room_name: RoomName,

  pub spawn_ids: Vec<RawObjectId>, // RawObjectId == ObjectId<StructureSpawn>
  pub extension_ids: Vec<RawObjectId>, // RawObjectId == ObjectId<StructureExtension>

  #[serde(skip)]
  available_spawn_cache: RefCell<CachedSpawnList>,
}

impl SpawnRoom {
  pub fn new(room: &Room) -> Self {
    let mut spawns = vec![];
    let mut extensions = vec![];

    for structure in room.find(find::MY_STRUCTURES) {
      match structure {
        StructureObject::StructureSpawn(spawn) => spawns.insert(0,spawn),
        StructureObject::StructureExtension(extension) => extensions.insert(0,extension),
        _ => {},
      }
    }

    Self {
      room_name: room.name(),

      spawn_ids: spawns.iter().map(|spawn| spawn.raw_id().into()).collect(),
      extension_ids: extensions.iter().map(|extension| extension.raw_id().into()).collect(),

      available_spawn_cache: RefCell::new(CachedSpawnList::default()),
    }
  }

  pub fn get_spawns(&self) -> Vec<StructureSpawn> {
    self.spawn_ids.iter().map(|spawn_id| ObjectId::<StructureSpawn>::from(spawn_id.to_owned()).resolve().unwrap()).collect()
  }

  pub fn get_extensions(&self) -> Vec<StructureExtension> {
    self.extension_ids.iter().map(|extension_id| ObjectId::<StructureExtension>::from(extension_id.to_owned()).resolve().unwrap()).collect()
  }

  pub fn available_energy(&self) -> u32 {
    if let Some(room) = game::rooms().get(self.room_name) {
      room.energy_available()
    } else {
      0
    }
  }

  pub fn max_energy(&self) -> u32 {
    if let Some(room) = game::rooms().get(self.room_name) {
      room.energy_capacity_available()
    } else {
      0
    }
  }

  pub fn available_spawn(&self) -> Option<StructureSpawn> {
    let mut cache = self.available_spawn_cache.borrow_mut();
    if cache.0 != game::time() {
      *cache = CachedSpawnList::default();
    }
    if cache.1.is_none() {
      let spawn_id_list = self.get_spawns().into_iter()
        .filter(|spawn| spawn.spawning().is_none())
        .map(|spawn| spawn.raw_id())
        .collect::<Vec<_>>();
      cache.1 = Some(RefCell::new(spawn_id_list));
    }
    match cache.1.as_ref() {
      Some(available_spawns) => {
        let selected_spawn = available_spawns.borrow_mut().pop();
        selected_spawn.map(|spawn_id| ObjectId::<StructureSpawn>::from(spawn_id.clone()).resolve()).flatten()
      },
      None => None,
    }
  }

  pub fn add_spawn(&mut self, spawn_id: ObjectId<StructureSpawn>) {
    self.spawn_ids.insert(0, spawn_id.into());
  }
}
use derivative::Derivative;
use screeps::{HasId, StructureObject, ObjectId, RawObjectId, StructureSpawn, StructureExtension, Room, find};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Derivative)]
#[derivative(Debug, Clone)]
pub struct SpawnGroup {
  pub id: usize,

  pub spawn_ids: Vec<RawObjectId>, // RawObjectId == ObjectId<StructureSpawn>
  pub extension_ids: Vec<RawObjectId>, // RawObjectId == ObjectId<StructureExtension>
}

impl SpawnGroup {
  pub fn new(id: usize, room: &Room) -> Self {
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
      id,

      spawn_ids: spawns.iter().map(|spawn| spawn.raw_id().into()).collect(),
      extension_ids: extensions.iter().map(|extension| extension.raw_id().into()).collect(),
    }
  }

  pub fn get_spawns(&self) -> Vec<StructureSpawn> {
    self.spawn_ids.iter().map(|spawn_id| ObjectId::<StructureSpawn>::from(spawn_id.clone()).resolve().unwrap()).collect()
  }

  pub fn get_extensions(&self) -> Vec<StructureExtension> {
    self.extension_ids.iter().map(|extension_id| ObjectId::<StructureExtension>::from(extension_id.clone()).resolve().unwrap()).collect()
  }

  // TODO : request a spawn with at least some amount of energy
  pub fn available_spawn(&self) -> Option<StructureSpawn> {
    self.get_spawns().into_iter().find(|spawn| spawn.spawning().is_none())
  }

  pub fn add_spawn(&mut self, spawn_id: ObjectId<StructureSpawn>) {
    self.spawn_ids.insert(0, spawn_id.into());
  }
}
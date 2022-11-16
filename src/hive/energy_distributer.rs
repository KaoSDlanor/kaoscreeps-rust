use screeps::{RawObjectId, RoomPosition, ObjectId, StructureSpawn, game, Creep, ReturnCode, SharedCreepProperties, ResourceType, HasId};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnergyDropOff {
  Spawn(RawObjectId), // RawObjectId == ObjectId<StructureSpawn>
  Creep(String), // String == CreepName
}

impl EnergyDropOff {
  pub fn preload(&self) -> Option<EnergyDropOffLoaded> {
    match self {
      EnergyDropOff::Spawn(spawn_id) => ObjectId::<StructureSpawn>::from(spawn_id.to_owned()).resolve().map(EnergyDropOffLoaded::Spawn),
      EnergyDropOff::Creep(creep_name) => game::creeps().get(creep_name.to_owned()).map(EnergyDropOffLoaded::Creep),
    }
  }
}

pub enum EnergyDropOffLoaded {
  Spawn(StructureSpawn),
  Creep(Creep),
}

impl EnergyDropOffLoaded {
  pub fn compress(&self) -> EnergyDropOff {
    match self {
      EnergyDropOffLoaded::Spawn(spawn) => EnergyDropOff::Spawn(spawn.raw_id()),
      EnergyDropOffLoaded::Creep(creep) => EnergyDropOff::Creep(creep.name()),
    }
  }

  pub fn pos(&self) -> RoomPosition {
    match self {
      Self::Spawn(spawn) => spawn.pos(),
      Self::Creep(creep) => creep.pos(),
    }
  }

  pub fn accept_energy(&self, hauler: &Creep, amount: Option<u32>) -> Result<ReturnCode,ReturnCode> {
    let return_code = match self {
      Self::Spawn(spawn) => hauler.transfer(spawn, ResourceType::Energy, amount),
      Self::Creep(creep) => hauler.transfer(creep, ResourceType::Energy, amount),
    };

    match return_code {
      ReturnCode::Ok => Ok(ReturnCode::Ok),
      failure_code => Err(failure_code),
    }
  }
}
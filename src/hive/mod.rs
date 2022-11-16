use std::{collections::HashMap};

use screeps::{game, Room, ReturnCode, Creep, Part, RoomName};
use serde::{Serialize, Deserialize};

pub mod energy_distributer;
pub mod id_generator;
pub mod mine_room;
pub mod tasks;
pub mod spawn_room;

// use id_generator::IdGenerator;
use mine_room::MineRoom;
use tasks::Tasks;
use spawn_room::SpawnRoom;

use self::energy_distributer::EnergyDropOffLoaded;

#[derive(Debug, Clone)]
pub enum GetCreepError {
  CreepBusy,
  NoSpawnAvailable,
  SpawningInProgress,
  SpawningFailed(ReturnCode),
}

impl From<GetCreepError> for String {
  fn from(get_creep_error: GetCreepError) -> Self {
    format!("{:?}",get_creep_error)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hive {
  // pub id_generator: IdGenerator,

  pub tasks: Tasks,

  pub mine_rooms: HashMap<RoomName,MineRoom>,
  pub spawn_rooms: HashMap<RoomName,SpawnRoom>,
}

impl Hive {
  pub fn new(room: Room) -> Self {
    // let mut id_generator = IdGenerator::new();

    let initial_spawn_group = SpawnRoom::new(&room);
    let energy_drop_off = EnergyDropOffLoaded::Spawn(initial_spawn_group.available_spawn().unwrap());
    let initial_mine_group = MineRoom::new(&room,room.name(),energy_drop_off);

    let mut spawn_rooms = HashMap::new();
    spawn_rooms.insert(room.name(),initial_spawn_group);

    let mut mine_rooms = HashMap::new();
    mine_rooms.insert(room.name(),initial_mine_group);

    Self {
      // id_generator,

      tasks: Tasks::new(),

      mine_rooms,
      spawn_rooms,
    }
  }

  pub fn get_creep<C>(&self, creep_name: String, spawn_room_name: &RoomName, urgent: bool, mut calculate_body: C) -> Result<Creep,GetCreepError>
  where C: FnMut(u32) -> Vec<Part> {
    if self.tasks.has_task(&creep_name) {
      return Err(GetCreepError::CreepBusy)
    }

    if let Some(creep) = game::creeps().get(creep_name.to_owned()) {
      if !creep.spawning() {
        return Ok(creep);
      } else {
        return Err(GetCreepError::SpawningInProgress);
      }
    }

    if let Some(spawn) = self.spawn_rooms.get(spawn_room_name).map(|spawn_group| spawn_group.available_spawn()).unwrap_or(None) {
      let energy = if urgent { spawn.room().unwrap().energy_available() } else { spawn.room().unwrap().energy_capacity_available() };
      let creep_body = calculate_body(energy);
      match spawn.spawn_creep(&creep_body, &creep_name) {
        ReturnCode::Ok => Err(GetCreepError::SpawningInProgress),
        unexpected => Err(GetCreepError::SpawningFailed(unexpected)),
      }
    } else {
      Err(GetCreepError::NoSpawnAvailable)
    }
  }

  pub fn run(&mut self) {
    for mine_group in self.mine_rooms.to_owned().values() {
      mine_group.run(self);
    }

    self.tasks.run();
  }
}
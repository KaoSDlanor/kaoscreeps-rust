use std::{collections::HashMap};

use screeps::{game, Room, ReturnCode, Creep, Part};
use serde::{Serialize, Deserialize};

pub mod id_generator;
pub mod mine_group;
pub mod tasks;
pub mod spawn_group;

use id_generator::IdGenerator;
use mine_group::MineGroup;
use tasks::Tasks;
use spawn_group::SpawnGroup;

pub enum GetCreepError {
  CreepBusy,
  NoSpawnAvailable,
  SpawningInProgress,
  SpawningFailed(ReturnCode),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hive {
  pub id_generator: IdGenerator,

  pub tasks: Tasks,

  pub mine_groups: HashMap<usize,MineGroup>,
  pub spawn_groups: HashMap<usize,SpawnGroup>,
}

impl Hive {
  pub fn new(room: Room) -> Self {
    let mut id_generator = IdGenerator::new();

    let initial_spawn_group = SpawnGroup::new(id_generator.generate(), &room);
    let initial_mine_group = MineGroup::new(id_generator.generate(), &room,initial_spawn_group.id);

    let mut spawn_groups = HashMap::new();
    spawn_groups.insert(initial_spawn_group.id,initial_spawn_group);

    let mut mine_groups = HashMap::new();
    mine_groups.insert(initial_mine_group.id,initial_mine_group);

    Self {
      id_generator,

      tasks: Tasks::new(),

      mine_groups,
      spawn_groups,
    }
  }

  pub fn run(&mut self) {
    self.tasks.run();

    for mine_group in self.mine_groups.clone().values() {
      mine_group.run(self);
    }
  }

  pub fn get_creep<C>(&self, creep_name: String, spawn_group_id: &usize, mut calculate_body: C) -> Result<Creep,GetCreepError>
  where C: FnMut(u32) -> Vec<Part> {
    if self.tasks.has_task(&creep_name) {
      return Err(GetCreepError::CreepBusy)
    }

    if let Some(creep) = game::creeps().get(creep_name.clone()) {
      if creep.spawning() == false {
        return Ok(creep);
      } else {
        return Err(GetCreepError::SpawningInProgress);
      }
    }

    if let Some(spawn) = self.spawn_groups.get(spawn_group_id).map(|spawn_group| spawn_group.available_spawn()).unwrap_or(None) {
      let creep_body = calculate_body(spawn.room().unwrap().energy_available());
      match spawn.spawn_creep(&creep_body, &creep_name) {
        ReturnCode::Ok => Err(GetCreepError::SpawningInProgress),
        unexpected => Err(GetCreepError::SpawningFailed(unexpected)),
      }
    } else {
      Err(GetCreepError::NoSpawnAvailable)
    }
  }
}
use screeps::{SharedCreepProperties, HasId, RawObjectId, Room, find, Source, Part, Creep, ObjectId};
use serde::{Serialize, Deserialize};

use crate::console;
use super::{Hive, GetCreepError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MineGroup {
  pub id: usize,
  pub spawn_group_id: usize,
  pub source_ids: Vec<RawObjectId>, // RawObjectId == ObjectId<Source>
}

impl MineGroup {
  pub fn new(id: usize,mine_room: &Room,spawn_group_id: usize) -> Self {
    Self {
      id,
      spawn_group_id,
      source_ids: mine_room.find(find::SOURCES).into_iter().map(|source| source.raw_id().into()).collect(),
    }
  }

  // TODO : remove MOVE part and use a tow service
  pub fn get_harvester_body(energy_available: u32) -> Vec<Part> {
      let req_work = screeps::constants::SOURCE_ENERGY_CAPACITY / screeps::constants::ENERGY_REGEN_TIME / 2;
      let max_addl_work = (energy_available - Part::Carry.cost() - Part::Move.cost() - Part::Work.cost()) / Part::Work.cost();

      let mut harvester_body = vec![Part::Move,Part::Carry,Part::Work];
      for _ in 0..max_addl_work.min(req_work - 1) {
        harvester_body.insert(harvester_body.len(), Part::Work);
      }
      harvester_body
  }

  pub fn get_harvester(&self, source: &Source, hive: &Hive) -> Option<Creep> {
    let harvester_name = String::from("harvester:") + &String::from(source.raw_id());
    match hive.get_creep(harvester_name.clone(), &self.spawn_group_id, Self::get_harvester_body) {
      Ok(harvester) => {
        return Some(harvester);
      },
      Err(GetCreepError::CreepBusy) => {
        console::warn(format!("[ mine_room  / {:?}] Unable to get harvester {:?} for source {:?}. Creep busy", self.id,harvester_name,(source.pos().room_name(),source.pos().x(),source.pos().y())));
      },
      Err(GetCreepError::SpawningInProgress) => {
        console::warn(format!("[ mine_room  / {:?}] Unable to get harvester {:?} for source {:?}. Creep still spawning", self.id,harvester_name,(source.pos().room_name(),source.pos().x(),source.pos().y())));
      },
      Err(GetCreepError::NoSpawnAvailable) => {
        console::warn(format!("[ mine_room / {:?} ] no spawn available for harvester {:?}", self.id, &harvester_name));
      },
      Err(GetCreepError::SpawningFailed(spawn_failed_reason)) => {
        console::warn(format!("[ mine_room / {:?} ] Failed to spawn harvester {:?} unexpected return code: {:?}", self.id, &harvester_name, spawn_failed_reason));
      }
    }
    None
  }

  // TODO : calculate required size based on how much energy needs to be hauled and how far it will be hauled
  pub fn get_hauler_body(_energy_available: u32) -> Vec<Part> {
    vec![Part::Move,Part::Carry,Part::Move,Part::Carry]
  }

  pub fn get_hauler(&self, source: &Source, hive: &Hive) -> Option<Creep> {
    let hauler_name = String::from("hauler:") + &String::from(source.raw_id());
    match hive.get_creep(hauler_name.clone(), &self.spawn_group_id, Self::get_hauler_body) {
      Ok(hauler) => {
        return Some(hauler);
      },
      Err(GetCreepError::CreepBusy) => {
        console::warn(format!("[ mine_room  / {:?}] Unable to get hauler {:?} for source {:?}. Creep busy", self.id,hauler_name,(source.pos().room_name(),source.pos().x(),source.pos().y())));
      },
      Err(GetCreepError::SpawningInProgress) => {
        console::warn(format!("[ mine_room / {:?} ] Unable to get hauler {:?} for source {:?}. Creep still spawning", self.id,hauler_name,(source.pos().room_name(),source.pos().x(),source.pos().y())));
      },
      Err(GetCreepError::NoSpawnAvailable) => {
        console::warn(format!("[ mine_room / {:?} ] no spawn available for hauler {:?}", self.id, &hauler_name));
      },
      Err(GetCreepError::SpawningFailed(spawn_failed_reason)) => {
        console::warn(format!("[ mine_room / {:?} ] Failed to spawn hauler {:?} unexpected return code: {:?}", self.id, &hauler_name, spawn_failed_reason));
      }
    }
    None
  }

  pub fn process_source(&self, harvester: Creep, hauler: Creep, source: Source) {
    match harvester.harvest(&source) {
      screeps::ReturnCode::Ok => {},
      screeps::ReturnCode::NotInRange => {
        match (hauler.pull(&harvester),harvester.move_pulled_by(&hauler)) {
          (screeps::ReturnCode::Ok,screeps::ReturnCode::Ok) => {
            // TODO : once you have reached the source pull the harvester into your spot
            match hauler.move_to(source) {
              screeps::ReturnCode::Ok => {},
              unexpected => {
                console::warn(format!("[ mine_room / {:?} ] Creep {:?} unexpected return code when moving to source: {:?}", self.id, hauler.name(), unexpected));
              },
            }
          },
          (screeps::ReturnCode::NotInRange,_) => {
            hauler.move_to(&harvester);
          },
          unexpected => {
            console::warn(format!("[ mine_room / {:?} ] Creep {:?} unexpected return code when pulling creep {:?}. Pull code: {:?}, Follow code: {:?}", self.id, hauler.name(), harvester.name(), unexpected.0, unexpected.1));
          },
        }
      },
      unexpected => {
        console::warn(format!("[ mine_room / {:?} ] Creep {:?} unexpected return code when harvesting: {:?}", self.id, harvester.name(), unexpected));
      },
    };
  }

  pub fn run(&self,hive: &Hive) {
    for source_id in self.source_ids.iter() {
      let source = ObjectId::<Source>::from(source_id.clone()).resolve().unwrap();
      if let (Some(harvester),Some(hauler)) = (self.get_harvester(&source, hive),self.get_hauler(&source, hive)) {
        self.process_source(harvester, hauler, source);
      }
    }
  }
}
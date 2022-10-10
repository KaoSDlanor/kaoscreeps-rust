use screeps::{SharedCreepProperties, HasId, RawObjectId, Room, find, Source, Part, Creep, ObjectId, RoomName, ReturnCode, ResourceType, look};
use serde::{Serialize, Deserialize};

use crate::console;
use super::{Hive, energy_distributer::{EnergyDropOff, EnergyDropOffLoaded}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MineRoom {
  pub room_name: RoomName,
  pub spawn_room_name: RoomName,
  pub energy_drop_off: EnergyDropOff,
  pub source_ids: Vec<RawObjectId>, // RawObjectId == ObjectId<Source>
}

impl MineRoom {
  pub fn new(room: &Room, spawn_room_name: RoomName, energy_drop_off: EnergyDropOffLoaded) -> Self {
    let drop_off_pos = energy_drop_off.pos();

    let mut sources = room.find(find::SOURCES)
      .into_iter()
      .map(|source| (source.pos().get_range_to(&drop_off_pos),source))
      .collect::<Vec<_>>();

    sources.sort_by(|a,b| a.0.cmp(&b.0));

    Self {
      room_name: room.name(),
      spawn_room_name,
      energy_drop_off : energy_drop_off.compress(),
      source_ids: sources
        .into_iter()
        .map(|entry| entry.1.raw_id())
        .collect(),
    }
  }

  // TODO : calculate required CARRY parts based on how much energy will be harvested between hauler visits and store energy in harvester if less is wasted
  pub fn get_harvester_body(energy_available: u32) -> Vec<Part> {
    let req_work = screeps::constants::SOURCE_ENERGY_CAPACITY / screeps::constants::ENERGY_REGEN_TIME / 2;
    let max_addl_work = (energy_available - Part::Work.cost()) / Part::Work.cost();

    let mut harvester_body = vec![Part::Work];
    for _ in 0..max_addl_work.min(req_work - 1) {
      harvester_body.insert(harvester_body.len(), Part::Work);
    }
    harvester_body
  }

  // TODO : calculate required size based on how much energy needs to be hauled and how far it will be hauled
  pub fn get_hauler_body(_energy_available: u32) -> Vec<Part> {
    vec![Part::Move,Part::Carry,Part::Move,Part::Carry]
  }

  fn tow_to_source(&self, source: Source, harvester: &Creep, hauler: &Creep) -> Result<(),String> {
    if hauler.pos().is_near_to(&harvester) {
      match (hauler.pull(&harvester),harvester.move_pulled_by(&hauler)) {
        (screeps::ReturnCode::Ok,screeps::ReturnCode::Ok) => {
          if hauler.pos().is_near_to(&source) {
            match hauler.move_pulled_by(&harvester) {
              screeps::ReturnCode::Ok => Ok(()),
              unexpected => {
                Err(format!("Creep {:?} unexpected return code when dumping harvester: {:?}", hauler.name(), unexpected))
              },
            }
          } else {
            match hauler.move_to(&source) {
              screeps::ReturnCode::Ok => Ok(()),
              unexpected => {
                Err(format!("Creep {:?} unexpected return code when moving to source: {:?}", hauler.name(), unexpected))
              },
            }
          }
        },
        unexpected => {
          Err(format!("Creep {:?} unexpected return code when pulling creep {:?}. Pull code: {:?}, Follow code: {:?}", hauler.name(), harvester.name(), unexpected.0, unexpected.1))
        },
      }
    } else {
      match hauler.move_to(&harvester) {
        ReturnCode::Ok => Ok(()),
        failure_reason => Err(format!("{:?}",failure_reason)),
      }
    }
  }

  fn mine_source(&self, source: Source, harvester: &Creep) -> Result<(),String> {
    match harvester.harvest(&source) {
      screeps::ReturnCode::Ok => Ok(()),
      failure_code => {
        Err(format!("Harvester {:?} unexpected return code when harvesting: {:?}", harvester.name(), failure_code))
      },
    }
  }

  fn haul_energy(&self, harvester: &Creep, hauler: &Creep) -> Result<(),String> {
    let hauler_store = hauler.store();
    if hauler_store.get_free_capacity(Some(ResourceType::Energy)) > 0 {
      if hauler.pos().is_near_to(&harvester) {
        // match harvester.transfer(hauler, ResourceType::Energy, None) {
        //   ReturnCode::Ok => Ok(()),
        //   failure_code => Err(format!("Harvester {:?} unexpected return code when transferring energy to hauler: {:?}", harvester.name(), failure_code)),
        // }
        match harvester.pos().look_for(look::ENERGY).get(0) {
          Some(resource) => {
            match hauler.pickup(resource) {
              ReturnCode::Ok => Ok(()),
              failure_code => Err(format!("Harvester {:?} unexpected return code when picking up energy: {:?}", harvester.name(), failure_code)),
            }
          },
          None => Err(format!("Hauler {:?} no energy to pick up", hauler.name()))
        }
      } else {
        match hauler.move_to(&harvester) {
          ReturnCode::Ok => Ok(()),
          failure_code => Err(format!("Hauler {:?} unexpected return code when approaching pickup: {:?}", hauler.name(), failure_code)),
        }
      }
    } else {
      match self.energy_drop_off.preload() {
        Some(drop_off) => {
          let drop_off_pos = drop_off.pos();
          if hauler.pos().is_near_to(&drop_off_pos) {
            drop_off.accept_energy(hauler, None)
              .map(|_| ())
              .map_err(|e| format!("Hauler {:?} unexpected return code when dropping off: {:?}", hauler.name(), e))
          } else {
            match hauler.move_to(drop_off_pos) {
              ReturnCode::Ok => Ok(()),
              failure_code => Err(format!("Hauler {:?} unexpected return code when approaching dropoff: {:?}", hauler.name(), failure_code)),
            }
          }
        },
        None => Err(format!("Hauler {:?} unable to resolve energy drop off {:?}", hauler.name(), self.energy_drop_off))
      }
    }
  }

  fn process_source(&self, source: Source, hive: &mut Hive) -> Result<(),String> {
    let hauler_name = String::from("hauler:") + &String::from(source.raw_id());
    let hauler_result = hive.get_creep(hauler_name.to_owned(), &self.spawn_room_name, false, Self::get_hauler_body);

    let harvester_name = String::from("harvester:") + &String::from(source.raw_id());
    let harvester = hive.get_creep(harvester_name.to_owned(), &self.spawn_room_name, false, Self::get_harvester_body)?;

    if harvester.pos().is_near_to(&source) {
      self.mine_source(source, &harvester)?;
      self.haul_energy(&harvester, &hauler_result?)?;
    } else {
      self.tow_to_source(source, &harvester, &hauler_result?)?;
    }

    Ok(())
  }

  pub fn run(&self,hive: &mut Hive) {
    for source_id in self.source_ids.iter() {
      let source = ObjectId::<Source>::from(source_id.to_owned()).resolve().unwrap();
      if let Err(failure_reason) = self.process_source(source, hive) {
        console::warn(format!("[ mine_room / {:?} ] Failed to process source {:?} because {:?}", self.room_name.to_string(), ObjectId::<Source>::from(source_id.to_owned()).to_string(), failure_reason));
      }
    }
  }
}
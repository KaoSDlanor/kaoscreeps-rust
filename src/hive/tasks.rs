use std::collections::{VecDeque, HashMap};

use screeps::{ReturnCode, Direction, ObjectId, Creep, Source, RawObjectId, game};
use serde::{Serialize, Deserialize};

pub enum TaskReturn {
  Complete,
  ProgressMade,
  Err(ReturnCode),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Task {
  Move(Direction),
  Tow(String,Direction), // String == Valid creep name
  Harvest(RawObjectId), // RawObjectId == ObjectId<Source>
  Continuous(Box<Task>),
  Perpetual(Box<Task>),
  MultiStep(VecDeque<Box<Task>>),
}

impl Task {
  pub fn run(&mut self, creep: Creep) -> TaskReturn {
    match self {

      Task::Move(direction) => {
        match creep.move_direction(direction.clone()) {
          ReturnCode::Ok => TaskReturn::Complete,
          return_code => TaskReturn::Err(return_code),
        }
      },

      Task::Tow(creep_name, direction) => {
        if let Some(towed) = game::creeps().get(creep_name.clone()) {
          match (creep.pull(&towed),towed.move_pulled_by(&creep)) {
            (ReturnCode::Ok,ReturnCode::Ok) => {
              match creep.move_direction(direction.clone()) {
                ReturnCode::Ok => TaskReturn::Complete,
                return_code => TaskReturn::Err(return_code),
              }
            },
            (return_code,_) if return_code != ReturnCode::Ok => TaskReturn::Err(return_code),
            (_,return_code) if return_code != ReturnCode::Ok => TaskReturn::Err(return_code),
            return_codes => panic!("Failed to handle return codes: {:?}, {:?}", return_codes.0, return_codes.1),
          }
        } else {
          TaskReturn::Err(ReturnCode::InvalidTarget)
        }
      },

      Task::Harvest(source_id) => {
        let source = ObjectId::<Source>::from(source_id.clone()).resolve().unwrap();
        match creep.harvest(&source) {
          ReturnCode::Ok => TaskReturn::Complete,
          return_code => TaskReturn::Err(return_code),
        }
      },

      Task::Continuous(task) => {
        match task.run(creep) {
          TaskReturn::Complete => TaskReturn::ProgressMade,
          other_return => other_return,
        }
      },

      Task::Perpetual(task) => {
        task.run(creep);
        TaskReturn::ProgressMade
      },

      Task::MultiStep(task_list) => {
        match task_list.front_mut() {
          Some(task) => {
            match task.run(creep) {
              TaskReturn::Complete => {
                task_list.pop_front();
                if task_list.len() > 0 {
                  TaskReturn::ProgressMade
                } else {
                  TaskReturn::Complete
                }
              },
              other_return => other_return,
            }
          },
          None => TaskReturn::Complete,
        }
      },

    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tasks {
  pub task_list: HashMap<String,Task>, // String == Valid creep name
}

impl Tasks {
  pub fn new() -> Self {
    Self {
      task_list: HashMap::new(),
    }
  }

  pub fn run(&mut self) {
    let mut completed_creep_names: Vec<String> = vec![];

    for (creep_name, task) in self.task_list.iter_mut() {
      if let Some(creep) = game::creeps().get(creep_name.clone().into()) {
        match task.run(creep) {
          TaskReturn::Complete => {
            completed_creep_names.push(creep_name.clone());
          },
          _ => {},
        }
      } else {
        completed_creep_names.push(creep_name.clone());
      }
    }

    for creep_name in completed_creep_names {
      self.task_list.remove(&creep_name);
    }
  }

  pub fn add_task(&mut self, creep_name: String, task: Task) {
    self.task_list.insert(creep_name, task);
  }

  pub fn has_task(&self, creep_name: &str) -> bool {
    self.task_list.contains_key(creep_name)
  }
}
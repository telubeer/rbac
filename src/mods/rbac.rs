extern crate json;
extern crate bodyparser;
extern crate serde_json;

use std::collections::{HashMap, HashSet};
use json::JsonValue;

pub type UserId = u32;
pub type ItemId = u16;

#[derive(Debug)]
pub struct Data {
    pub map: HashMap<String, ItemId>,
    pub assignments: HashMap<UserId, HashSet<ItemId>>,
    pub assignments_dict: HashMap<ItemId, Assignment>,
    pub items: HashMap<ItemId, Item>,
    pub parents: HashMap<UserId, HashMap<ItemId, HashSet<ItemId>>>
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: ItemId,
    pub data: json::JsonValue,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub user_id: UserId,
    pub name: ItemId,
    pub data: json::JsonValue,
}

impl Data {
    pub fn new() -> Self {
        Data {
            assignments: HashMap::new(),
            assignments_dict: HashMap::new(),
            items: HashMap::new(),
            parents: HashMap::new(),
            map: HashMap::new()
        }
    }

    pub fn check_access(&self, user_id: UserId, action: String, params: &JsonValue) -> bool {
        if let Some(item_id) = self.map.get(&action) {
            if let Some(assignments) = self.assignments.get(&user_id) {
                return self.check(&user_id, item_id.clone(), &assignments, params);
            }
        }
        return false;
    }

    /**
    *   54ns
    **/
    pub fn rule(&self, data: &JsonValue, params: &JsonValue) -> bool {
        if let Some(key) = data["paramsKey"].as_str() {
            if let Some(value) = params[key].as_str() {
                if data["data"].is_array() {
                    return data["data"].contains(value);
                } else {
                    return true;
                }
            } else {
                return false;
            }
        } else {
            return true;
        }
    }

    fn check(&self, user_id: &UserId, action: ItemId, assignments: &HashSet<ItemId>, params: &JsonValue) -> bool {
        match self.items.get(&action) {
            Some(item) => {
                if !self.rule(&item.data, params) {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }
        if assignments.contains(&action) {
            if self.rule(&self.assignments_dict.get(&action).unwrap().data, params) {
                return true;
            }
        }
        if let Some(user_parents) = self.parents.get(user_id) {
            if let Some(parents) = user_parents.get(&action) {
                for parent in parents {
                    if self.check(user_id, parent.clone(), &assignments, params) {
                        return true;
                    }
                }
            }
        }

        return false;
    }
}

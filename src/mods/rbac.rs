extern crate json;
extern crate bodyparser;
extern crate serde_json;

use std::collections::{HashMap, HashSet};
use json::JsonValue;

#[derive(Debug)]
pub struct Data {
    pub assignments: HashMap<String, HashSet<String>>,
    pub assignments_dict: HashMap<String, Assignment>,
    pub items: HashMap<String, Item>,
    pub parents: HashMap<String, Vec<String>>
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub rule: Option<String>,
    pub data: json::JsonValue,
    pub item_type: i64,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub user_id: String,
    pub name: String,
    pub rule: Option<String>,
    pub data: json::JsonValue,
}

impl Data {
    pub fn new() -> Self {
        Data {
            assignments: HashMap::new(),
            assignments_dict: HashMap::new(),
            items: HashMap::new(),
            parents: HashMap::new()
        }
    }

    pub fn check_access(&self, user_id: String, action: String, params: &JsonValue) -> bool {
        if let Some(assignments) = self.assignments.get(&user_id) {
            return self.check(action, &assignments, params);
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

    fn check(&self, action: String, assignments: &HashSet<String>, params: &JsonValue) -> bool {
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

        if let Some(parents) = self.parents.get(&action) {
            for parent in parents {
                if self.check(parent.to_string(), &assignments, params) {
                    return true;
                }
            }
        }
        return false;
    }
}


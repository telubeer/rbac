#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]
#![feature(test)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::State;
use std::sync::{Mutex, MutexGuard};
use std::env;

extern crate test;
extern crate mysql;
#[macro_use]
extern crate json;
extern crate time;

mod mods;
mod tests;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rocket_contrib::Json;
use time::precise_time_ns;
use mods::rbac::{Assignment, Data, Item};
use mods::phpdeserializer::Deserializer;
use mysql as my;
use serde_json::Value as JsonValue;
use std::collections::{HashSet, HashMap};

#[derive(Serialize, Deserialize, Debug)]
struct Params {
    user_id: String,
    action: String,
    params: Vec<JsonValue>
}

#[derive(Serialize, Deserialize, Debug)]
struct Out {
    result: bool
}

#[post("/", format = "application/json", data = "<body>")]
fn check(body: Json<Vec<Params>>, data: State<Mutex<Data>>) -> Json<Vec<Vec<bool>>> {
    let rbac: MutexGuard<Data> = data.lock().expect("map lock.");
    let mut out = Vec::new();
    for item in body.iter() {
        let mut r = Vec::new();
        for param in item.params.iter() {
            let action = item.action.clone();
            let user = item.user_id.clone();
            r.push(rbac.check_access(user, action, &param))
        }
        out.push(r);
    }
    Json(out)
}

fn main() {
    let data = load();
    println!("данные загружены");
    rocket::ignite()
        .mount("/check", routes![check])
        .manage(Mutex::new(data))
        .launch();
}

fn load() -> Data {
    let (mut items, mut parents, mut assignments) = load_items();

    let mut data = Data::new();
    while items.len() > 0 {
        let item = items.pop().unwrap();
        let name = item.name.clone();
        data.items.insert(name, item);
    }
    while parents.len() > 0 {
        let (parent, child) = parents.pop().unwrap();
        if !data.parents.contains_key(&child) {
            let c = child.clone();
            data.parents.insert(c, Vec::new());
        }
        let vec = data.parents.get_mut(&child).unwrap();
        vec.push(parent);
    }
    while assignments.len() > 0 {
        let assignment = assignments.pop().unwrap();
        let user = assignment.user_id.clone();
        let name = assignment.name.clone();
        if !data.assignments.contains_key(&user) {
            let u = user.clone();
            data.assignments.insert(u, HashSet::new());
        }
        let hashmap = data.assignments.get_mut(&user).unwrap();
        hashmap.insert(name);
        let name = assignment.name.clone();
        data.assignments_dict.insert(name, assignment);
    }
    return data;
}

fn load_items() -> (Vec<Item>, Vec<(String, String)>, Vec<Assignment>) {
    let bind_to = env::var("DSN").ok()
        .expect("You should set mysql connection settings mysql://user:pass@ip:port in DSN env var");
    let pool = my::Pool::new(&bind_to).unwrap();
    let items: Vec<Item> =
        pool.prep_exec("SELECT name, biz_rule as rule, data, type as item_type from ngs_regionnews.auth_item", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|mut row| {
                    let data: String = row.take("data").unwrap();
                    let mut d = Deserializer::from_str(&data);
                    Item {
                        name: row.take("name").unwrap(),
                        rule: row.take("rule").unwrap(),
                        data: d.parse(),
                        item_type: row.take("item_type").unwrap(),
                    }
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap();

    let parents: Vec<(String, String)> =
        pool.prep_exec("SELECT parent, child from ngs_regionnews.auth_item_child  ORDER BY parent DESC", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    let (parent, child) = my::from_row(row);
                    return (parent, child);
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap();

    let assignments: Vec<Assignment> =
        pool.prep_exec("SELECT user_id, item_name as name, biz_rule as rule, data from ngs_regionnews.auth_assignment", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|mut row| {
                    let data: String = row.take("data").unwrap();
                    let mut d = Deserializer::from_str(&data);
                    Assignment {
                        user_id: row.take("user_id").unwrap(),
                        name: row.take("name").unwrap(),
                        rule: row.take("rule").unwrap(),
                        data: d.parse(),
                    }
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap();
    return (items, parents, assignments);
}
#![feature(test)]
extern crate test;
extern crate mysql;
#[macro_use] extern crate json;

mod mods;
mod tests;

use mods::rbac::{Assignment, Data, Item};
use mods::phpdeserializer::Deserializer;
use mods::server::run;
use mysql as my;
use std::collections::HashSet;
use std::{env};


fn main() {
    let bind_to = env::var("BIND").ok()
            .expect("You should set ip:port in BIND env var");
    let data = load();
    println!("data loaded");
    run(&bind_to, data);
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
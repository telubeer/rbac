extern crate mysql;
extern crate time;
use mods::rbac::{Assignment, Data, Item, ItemId, UserId};
use mods::phpdeserializer::Deserializer;
use std::collections::{HashSet, HashMap};
use self::time::precise_time_ns;
use mysql::Pool;

pub fn load(pool: &Pool) -> Data {
    let (map, mut items, mut parents, mut assignments) = load_items(pool);
//    let start = precise_time_ns();

    let mut data = Data::new();
    data.map = map;

    while items.len() > 0 {
        let item = items.pop().unwrap();
        let name = item.name.clone();
        data.items.insert(name, item);
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

    let mut children: HashMap<ItemId, Vec<ItemId>> = HashMap::new();

    while parents.len() > 0 {
        let (parent, child) = parents.pop().unwrap();

        if !children.contains_key(&parent) {
            children.insert(parent.clone(), Vec::new());
        }
        let vec = children.get_mut(&parent).unwrap();
        vec.push(child);
    }

//    let start1 = precise_time_ns();
    for assignment in data.assignments.clone().iter() {
        let (user_id, roles) = assignment;
        if !data.parents.contains_key(user_id) {
            data.parents.insert(user_id.clone(), HashMap::new());
        }
        for role in roles.iter() {
            process_childs(&user_id, &role, &mut data, &children);
        }
    }

//    println!("parse childs {} ms", (precise_time_ns() - start1)/ 1000000);
//    println!("load time {} ms", (precise_time_ns() - start)/ 1000000);
    return data;
}

fn process_childs(user_id: &UserId, parent: &ItemId, data: &mut Data, children: &HashMap<ItemId, Vec<ItemId>>) {
    if !children.contains_key(parent) {
        return;
    }
    for child in children.get(parent).unwrap().iter() {
        if !data.parents[user_id].contains_key(child) {
            data.parents.get_mut(user_id).unwrap().insert(child.clone(), HashSet::new());
        }
        data.parents.get_mut(user_id).unwrap().get_mut(child).unwrap().insert(parent.clone());
        process_childs(user_id, child, data, children);
    }
}

fn load_items(pool: &Pool) -> (HashMap<String, ItemId>, Vec<Item>, Vec<(ItemId, ItemId)>, Vec<Assignment>) {
//    let start = precise_time_ns();
    let mut counter:ItemId = 0;
    let mut map: HashMap<String, ItemId> = HashMap::new();
    let items: Vec<Item> =
        pool.prep_exec("SELECT name, biz_rule as rule, data, type as item_type from ngs_regionnews.auth_item", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|mut row| {
                    let data: String = row.take("data").unwrap();
                    let mut d = Deserializer::from_str(&data);
                    let name:String = row.take("name").unwrap();
                    if !map.contains_key(&name) {
                        counter += 1;
                        map.insert(name.clone(), counter.clone());
                    }
                    Item {
                        name: map.get(&name).unwrap().clone(),
//                        rule: row.take("rule").unwrap(),
                        data: d.parse(),
//                        item_type: row.take("item_type").unwrap(),
                    }
                }).collect()
            }).unwrap();



    let assignments: Vec<Assignment> =
        pool.prep_exec("SELECT user_id, item_name as name, biz_rule as rule, data from ngs_regionnews.auth_assignment", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|mut row| {
                    let data: String = row.take("data").unwrap();
                    let mut d = Deserializer::from_str(&data);
                    let name: String = row.take("name").unwrap();
                    if !map.contains_key(&name) {
                        counter += 1;
                        map.insert(name.clone(), counter.clone());
                    }
                    let user: String = row.take("user_id").unwrap();
                    let user_id: UserId = user.parse().unwrap();
                    Assignment {
                        user_id,
                        name: map.get(&name).unwrap().clone(),
//                        rule: row.take("rule").unwrap(),
                        data: d.parse(),
                    }
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap();

    let parents: Vec<(ItemId, ItemId)> =
        pool.prep_exec("SELECT parent, child from ngs_regionnews.auth_item_child  ORDER BY parent DESC", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|mut row| {
                    let parent:String = row.take("parent").unwrap();
                    let child:String = row.take("child").unwrap();
                    return (map.get(&parent).unwrap().clone(), map.get(&child).unwrap().clone());
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap();
//    println!("fetch time {} ms", (precise_time_ns() - start)/ 1000000);
    return (map, items, parents, assignments);
}
#![feature(test)]
extern crate test;
#[macro_use] extern crate json;
#[macro_use] extern crate mysql;




mod mods;
mod tests;
use mysql::Pool;
use mods::server::run;
use mods::loader::load;
use std::env;

fn main() {
    let bind_to = env::var("BIND").ok()
        .expect("You should set ip:port in BIND env var");
    let dsn = env::var("DSN").ok()
        .expect("You should set mysql connection settings mysql://user:pass@ip:port in DSN env var");
    let mut pool = Pool::new(&dsn).unwrap();
    let data = load(&pool);
    println!("data loaded {:?}", data.assignments.len());
    run(&bind_to, data, pool);
}


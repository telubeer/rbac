//#![feature(test,libc)]

#[macro_use] extern crate json;
#[macro_use] extern crate mysql;
#[macro_use] extern crate serde_derive;
extern crate toml;

mod mods;
//mod tests;
use mysql::Pool;
use mods::server::run;
use mods::loader::load;
use mods::config::load_config;

fn main() {
    let config = load_config();
    let bind_to = config.get_bind();
    let dsn = config.get_dsn();
    let mut pool = Pool::new(&dsn).unwrap();
    let data = load(&pool);
    println!("loaded rules for {:?} users", data.assignments.len());
    run(&bind_to, data, pool);
}




extern crate mysql;
extern crate rbac;

use mysql::Pool;
use rbac::mods::server::run;
use rbac::mods::loader::load;
use rbac::mods::config::load_config;

fn main() {
    let config = load_config();
    let bind_to = config.get_bind();
    let dsn = config.get_dsn();
    let pool = Pool::new(&dsn).unwrap();
    let data = load(&pool);
    println!("loaded rules for {:?} users", data.assignments.len());
    run(&bind_to, data, pool);
}




//#![feature(test,libc)]
#![feature(test)]
extern crate test;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate mysql;
//extern crate libc;
//extern {fn je_stats_print (write_cb: extern fn (*const libc::c_void, *const libc::c_char), cbopaque: *const libc::c_void, opts: *const libc::c_char);}
//extern fn write_cb (_: *const libc::c_void, message: *const libc::c_char) {
//    print! ("{}", String::from_utf8_lossy (unsafe {std::ffi::CStr::from_ptr (message as *const i8) .to_bytes()}));}


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
    pool.use_cache(false);
//    unsafe {je_stats_print (write_cb, std::ptr::null(), std::ptr::null())};
    let data = load(&pool);
//    unsafe {je_stats_print (write_cb, std::ptr::null(), std::ptr::null())};
    println!("loaded rules for {:?} users", data.assignments.len());
    run(&bind_to, data, pool);
}


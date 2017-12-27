extern crate rbac;
extern crate mysql;
extern crate tokio_core;
extern crate futures;
extern crate persistent;
#[macro_use] extern crate log;
extern crate env_logger;

use mysql::Pool;
use std::sync::{Arc, RwLock};
//use rbac::mods::server::run;
use rbac::mods::gotham::start;
use rbac::mods::loader::{load, get_timestamp};
use rbac::mods::config::load_config;
//use rbac::mods::rbac::Data;
use std::thread;
use std::time::Duration;
use tokio_core::reactor::{Core, Interval, Remote};
use futures::{Future, Stream, Sink};
use futures::sync::mpsc::{channel, Receiver, Sender};


fn main() {
    env_logger::init().expect("Failed to initialize logger");
    let config = load_config();
    let bind_to = config.get_bind();
    let dsn = config.get_dsn();
    let pool = Pool::new(&dsn).expect("Failed to initialize db pool");

    let data = load(&pool, get_timestamp(&pool));
    info!("loaded rules for {:?} users", data.assignments.len());

    let data_arc = Arc::new(RwLock::new(data));
    let pool_arc = Arc::new(RwLock::new(pool));

    let (tx, rx): (Sender<i8>, Receiver<i8>) = channel(100);

    let mut worker_core = Core::new().expect("Failed to initialize main event loop");

    run_timer(config.get_timer(),tx.clone(), worker_core.remote());

    let data_arc_server = data_arc.clone();
//    let remote_server = worker_core.remote();
    thread::spawn(move || {
        info!("spawned server thread");
        start(bind_to, data_arc_server);
//        run(&bind_to, data_arc_server, tx.clone(), remote_server);
    });

    let data_worker = data_arc.clone();
    worker_core.run(rx.for_each(move |item| {
        info!("spawned worker thread");
        let mut timestamp = 0;
        let need_reload = match item {
            1 => {
                timestamp = get_timestamp(&pool_arc.read().unwrap());
                let data_read = &data_worker.read().unwrap();
                timestamp != data_read.timestamp
            }
            2 => {
                true
            }
            _ => {
                false
            }
        };

        if need_reload {
            debug!("do reload by request - start");
            let data = load(&pool_arc.read().unwrap(), timestamp);
            let mut data_write = data_worker.write().unwrap();
            *data_write = data;
            debug!("do reload by request - done");
        }

        Ok(())
    })).expect("Failed to spawn worker thread");
}


fn run_timer(timer: u64, tx_timer: Sender<i8>, remote_timer: Remote) {
    thread::spawn(move || {
        info!("spawned timer thread");
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let interval_stream = Interval::new(Duration::new(timer, 0), &handle)
            .unwrap()
            .map(|()| {
                let tx_timer = tx_timer.clone();
                remote_timer.spawn(move |_| {
                    tx_timer.send(1)
                        .then(|tx| {
                            match tx {
                                Ok(_tx) => {
                                    debug!("send timer");
                                    Ok(())
                                }
                                Err(e) => {
                                    error!("send timer failed! {:?}", e);
                                    Err(())
                                }
                            }
                        })
                })
            });

        core.run(interval_stream.for_each(move |()| {
            Ok(())
        })).unwrap();
    });
}

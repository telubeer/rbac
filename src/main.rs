extern crate rbac;
extern crate mysql;
extern crate tokio;
extern crate futures;
#[macro_use] extern crate log;
extern crate env_logger;

use mysql::Pool;
use std::sync::{Arc, RwLock};
use rbac::mods::server::run;
use rbac::mods::loader::{load, get_timestamp};
use rbac::mods::config::load_config;
use rbac::mods::rbac::Data;
use std::thread;
use std::time::{Duration, Instant};
use tokio::timer::Interval;
use tokio::runtime::current_thread::{Runtime};
use futures::{Future, Stream, Sink};
use futures::sync::mpsc::{channel, Receiver, Sender};
use rbac::mods::config::Config;


fn main() {
    env_logger::init();
    let config = load_config();
    let bind_to = config.get_bind();
    let dsn = config.get_dsn();
    let pool = Pool::new(&dsn).expect("Failed to initialize db pool");

    let data = load(&pool, get_timestamp(&pool, &config), &config);
    info!("loaded rules for {:?} users", data.assignments.len());

    let data_arc = Arc::new(RwLock::new(data));

    let(tx,rx) = channel::<u8>(2);
    let workers = config.get_workers();
    let mut runtime = Runtime::new()
        .expect("create runtime failed");
    runtime.spawn(get_timer(config.get_timer(), tx.clone()));
    runtime.spawn(get_worker(rx, pool, data_arc.clone(), config));

    let handle = runtime.handle();
    thread::spawn(move || {
        info!("spawned server thread");
        run(&bind_to, data_arc.clone(), tx.clone(), handle, workers);
    });

    runtime
        .run()
        .expect("runtime start failed");
}

fn get_worker(rx: Receiver<u8>, pool: Pool, data_arc: Arc<RwLock<Data>>, config: Config)
    -> impl Future<Item=(), Error=()>
{
    rx
        .for_each(move|event| {
            match pool.get_conn() {
                Ok(_conn) => {
                    let timestamp = get_timestamp(&pool, &config);
                    let need_reload = match event {
                        1 => {
                            let data_read = &data_arc.read().unwrap();
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
                        info!("do reload by request - start event={:?}", event);
                        let data = load(&pool, timestamp, &config);
                        let mut data_write = data_arc.write().unwrap();
                        *data_write = data;
                        info!("do reload by request - done");
                    }
                },
                Err(e) => {
                    error!("db connection error {:?}", e);
                }
            };
            Ok(())
        })
        .map_err(|e| error!("channel reciever err={:?}", e))
}

fn get_timer(duration: u64, tx_worker: Sender<u8>)
    -> impl Future<Item=(), Error=()>
{
    Interval::new(
        Instant::now(),
        Duration::from_secs(duration)
    )
        .for_each(move|instant| {
            match tx_worker.clone().send(1).wait() {
                Ok(_) => info!("timer event sended {:?}", instant),
                Err(e) => error!("send event err={:?}", e)
            };
            Ok(())
        })
        .map_err(|e| { error!("timer err={:?}", e) })
}

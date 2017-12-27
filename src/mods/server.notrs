extern crate json;
extern crate iron;
extern crate bodyparser;
extern crate persistent;
extern crate router;
extern crate time;
extern crate sys_info;
extern crate futures;
extern crate tokio_core;

use json::JsonValue;
use self::iron::prelude::*;
use self::iron::headers::ContentType;
use self::iron::{status, Listening};
use self::persistent::{Read};
use std::sync::{Arc, RwLock};
use self::iron::typemap::Key;
use self::router::Router;
use self::time::now;
use self::futures::sync::mpsc::Sender;
use self::futures::{Future, Sink};
use self::tokio_core::reactor::{Remote};

use mods::rbac::{Data, UserId};

header! { (XDataTimestamp, "X-Data-Timestamp") => [u32] }

#[derive(Copy, Clone)]
pub struct DataArc;

impl Key for DataArc {
    type Value = Arc<RwLock<Data>>;
}

#[derive(Copy, Clone)]
pub struct Uptime;

impl Key for Uptime { type Value = i64; }

#[derive(Copy, Clone)]
pub struct Tx;

impl Key for Tx { type Value = Sender<i8>; }

#[derive(Copy, Clone)]
pub struct Rem;

impl Key for Rem { type Value = Remote; }

fn handle(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<Read<DataArc>>().unwrap();
    let data = arc.as_ref().read().unwrap();

    let mut out: JsonValue = array![];
    let body = req.get::<bodyparser::Raw>();
    match body {
        Ok(Some(body)) => {
            let items = json::parse(&body).unwrap();
            for item in items.members() {
                let user_id: UserId = item["user_id"].to_string().parse().unwrap();
                let action = &item["action"];
                let params = &item["params"];
                let mut res: JsonValue = array![];
                for param in params.members() {
                    let result = data.check_access(
                        user_id,
                        action.to_string(),
                        &param
                    );
                    let _ = res.push(result);
                }
                let _ = out.push(res);
            }
        }
        Ok(None) => error!("No body"),
        Err(err) => error!("Error: {:?}", err)
    }
    let mut res = Response::with((ContentType::json().0, status::Ok, json::stringify(out)));
    res.headers.set(XDataTimestamp(data.timestamp.to_owned()));
    Ok(res)
}

fn reload(req: &mut Request) -> IronResult<Response> {
    let tx_arc = &req.get::<Read<Tx>>().unwrap();
    let tx = tx_arc.as_ref().clone();
    let remote_arc = &req.get::<Read<Rem>>().unwrap();
    let remote = remote_arc.as_ref();
    remote.spawn(move|_| {
        tx.send(2)
            .then(|tx| {
                match tx {
                    Ok(_tx) => {
                        debug!("send work");
                        Ok(())
                    }
                    Err(e) => {
                        error!("send work failed! {:?}", e);
                        Err(())
                    }
                }
            })
    });

    let data = object! {
        "status" => "ok",
    };
    Ok(Response::with((ContentType::json().0, status::Ok, json::stringify(data))))
}

fn health(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<Read<DataArc>>().unwrap();
    let data = arc.as_ref().read().unwrap();
    let start_time = &req.get::<Read<Uptime>>().unwrap();
    let uptime = now().to_timespec().sec - start_time.as_ref();
    let hostname = self::sys_info::hostname().unwrap();
    let data = object! {
        "status" => "ok",
        "uptime" => uptime,
        "hostname" => hostname,
        "data_timestamp" => data.timestamp
    };
    Ok(Response::with((ContentType::json().0, status::Ok, json::stringify(data))))
}

pub fn run(listen: &str, data: Arc<RwLock<Data>>, tx: Sender<i8>, remote: Remote) -> Listening {
    let start_time = now().to_timespec().sec;
    let mut router = Router::new();
    router.post("/check", handle, "check");
    router.post("/reload", reload, "reload");
    router.get("/health", health, "health");

    let mut chain = Chain::new(router);
    chain.link(Read::<DataArc>::both(data));
    chain.link(Read::<Uptime>::both(start_time));
    chain.link(Read::<Tx>::both(tx));
    chain.link(Read::<Rem>::both(remote));

    println!("start listening on {} hostname {}", &listen, self::sys_info::hostname().unwrap());
    Iron::new(chain).http(listen).unwrap()
}

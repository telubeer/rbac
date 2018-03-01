use net2;
use net2::unix::UnixTcpBuilderExt;

use std::thread;
use std::net::SocketAddr;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Remote;

use futures;
use futures::Poll;
use futures::future::FutureResult;
use futures::{Future, Stream};
use futures::sync::mpsc::Sender;

extern crate time;
extern crate futures_cpupool;
extern crate num_cpus;

use json;

use std::sync::{Arc, RwLock};

use mods::rbac::{Data, UserId};
use json::JsonValue;
use hyper;
use hyper::{Post, Get, StatusCode};
use hyper::header::ContentLength;
use hyper::header::{Headers, ContentType};
use hyper::server::{Http, Request, Response, Service};
use std::str;
use self::futures_cpupool::CpuPool;

#[derive(Clone)]
struct WebService {
    thread_pool: CpuPool,
    data: Arc<RwLock<Data>>,
}

impl WebService {
    pub fn new(thread_pool: CpuPool, data: Arc<RwLock<Data>>) -> WebService {
        WebService {
            thread_pool,
            data: data.clone(),
        }
    }
}

impl Service for WebService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResponse;

    fn call(&self, req: Request) -> Self::Future {
        let data = self.data.clone();
        let threadp = self.thread_pool.clone();
        let fr = match (req.method(), req.path()) {
            (&Post, "/check") => {
                let r =
                    req.body().concat2()
                        .map(|raw_body| {
                            str::from_utf8(&raw_body).unwrap().to_string()
                        })
                        .map(move |body| {
                            match json::parse(&body) {
                                Ok(items) => {
                                    let mut out: JsonValue = array![];
                                    for item in items.members() {
                                        let user_id: UserId = item["user_id"].to_string().parse().unwrap();
                                        let action = &item["action"];
                                        let params = &item["params"];
                                        let mut res: JsonValue = array![];
                                        for param in params.members() {
                                            let result = data.read().unwrap()
                                                .check_access(
                                                    user_id,
                                                    action.to_string(),
                                                    &param,
                                                );
                                            let _ = res.push(result);
                                        }
                                        let _ = out.push(res);
                                    }
                                    Response::new().with_body(json::stringify(out))
                                }
                                Err(e) => {
                                    error!("bad request {}", e);
                                    Response::new().with_status(StatusCode::BadRequest)
                                }
                            }
                        });
                FutureResponse(Box::new(r))
            }
            _ => {
                let res = Response::new().with_status(StatusCode::NotFound);
                FutureResponse(Box::new(futures::finished(res)))
            }
        };
        fr
    }
}


pub struct FutureResponse(Box<Future<Item=Response, Error=hyper::Error>>);

impl Future for FutureResponse {
    type Item = Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}


pub fn run(listen: &str, data: Arc<RwLock<Data>>, _tx: Sender<i8>, _remote: Remote) {
    let addr = listen.to_string().parse().unwrap();
    let num = num_cpus::get();
    let protocol = Arc::new(Http::new());
    for i in 0..num - 1 {
        println!("spawn {:?}", i);
        let protocol2 = protocol.clone();
        let data_arc = data.clone();
        thread::spawn(move || serve(&addr, &protocol2, data_arc));
    }
    println!("spawn {:?}", num);
    serve(&addr, &protocol, data.clone());
}

fn serve(addr: &SocketAddr,
         protocol: &Http,
         data: Arc<RwLock<Data>>) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let thread_pool = CpuPool::new(2);

    let listener = net2::TcpBuilder::new_v4()
        .unwrap()
        .reuse_port(true)
        .unwrap()
        .bind(addr)
        .unwrap()
        .listen(50)
        .unwrap();

    let listener = TcpListener::from_listener(listener, addr, &handle).unwrap();
    core.run(listener
        .incoming()
        .for_each(|(socket, addr)| {
            let s = WebService::new(thread_pool.clone(), data.clone());
            protocol.bind_connection(&handle, socket, addr, s);
            Ok(())
        })
        .or_else(|e| -> FutureResult<(), ()> {
            panic!("TCP listener failed: {}", e);
        }))
        .unwrap();
}





















/*






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
*/
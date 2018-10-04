use std::net::SocketAddr;
use tokio::runtime::current_thread::Handle;
use futures::{future, Future, Stream, Sink};
use futures::sync::mpsc::Sender;
use json;
use std::sync::{Arc, RwLock};
use mods::rbac::{Data, UserId};
use json::JsonValue;
use hyper;
use hyper::service::{NewService, Service};
use hyper::{Body,Error,Response,StatusCode,Request,Server,Method, Chunk};
use std::str;

type CEvent = u8;

#[derive(Clone)]
struct WebService {
    data: Arc<RwLock<Data>>,
    tx: Sender<CEvent>,
    remote: Handle,
}

impl WebService {
    pub fn new(data: Arc<RwLock<Data>>, tx: Sender<CEvent>, remote: Handle) -> WebService {
        WebService {
            data: data.clone(),
            tx: tx.clone(),
            remote: remote.clone(),
        }
    }
}

impl NewService for WebService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Service = WebService;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type InitError = hyper::Error;

    fn new_service(&self) -> Self::Future {
        Box::new(future::ok(self.clone()))
    }
}

impl Service for WebService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = Box<Future<Item = Response<Body>, Error = Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let data = self.data.clone();
        let remote = self.remote.clone();
        let tx = self.tx.clone();
        let mut response = Response::new(Body::empty());
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/reload") => {
                remote.spawn(
                    tx.send(2).then(|_| {
                        Ok(())
                    })
                ).unwrap();
                *response.status_mut() = StatusCode::OK;
            }
            (&Method::GET, "/health") => {
                let out = object! {
                    "timestamp" => data.read().unwrap().timestamp
                };
                *response.body_mut() = Body::from(json::stringify(out));
                *response.status_mut() = StatusCode::OK;
            }
            (&Method::POST, "/check") => {
                let r = req.into_body().concat2()
                    .map(move|chunk: Chunk| {
                        let body = str::from_utf8(&chunk.into_bytes()).unwrap().to_string();
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
                                *response.body_mut() = Body::from(json::stringify(out));
                                *response.status_mut() = StatusCode::OK;
                                response
                            }
                            Err(e) => {
                                error!("bad request {}", e);
                                *response.status_mut() = StatusCode::BAD_REQUEST;
                                response
                            }
                        }
                    });
                return Box::new(r);
            }
            _ => {
                *response.status_mut() = StatusCode::BAD_REQUEST;
            }
        }
        Box::new(future::ok(response))
    }
}


pub fn run(listen: &str,
           data: Arc<RwLock<Data>>,
           tx: Sender<CEvent>,
           remote: Handle,
           _workers: u8) {
    let addr = listen.to_string().parse().unwrap();
    serve(&addr, data.clone(), tx.clone(), remote.clone());
}

fn serve(addr: &SocketAddr,
         data: Arc<RwLock<Data>>,
         tx: Sender<CEvent>,
         remote: Handle) {

    let server = Server::bind(&addr)
        .serve(move || WebService::new(data.clone(), tx.clone(), remote.clone()).new_service())
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
